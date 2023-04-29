use std::{cmp::max, collections::HashMap, sync::Arc};

use crate::{chat, GetUpdatesRequest, TelegramClient, API};

// Handler routing:
//  - by chat ID
//  - by user
//  - by message type
//  - by message text regex
//
// create filtering functions for each of these, and then compose them together

pub struct Router<R, S, T>
where
    R: Into<chat::Action<chat::Op>>,
    T: TelegramClient,
{
    api: Arc<API<T>>,
    chat_handlers: Vec<chat::Handler<R, S, T>>,
    chat_state: HashMap<i64, S>,
}

impl<R: Into<chat::Action<chat::Op>>, S: Clone + Default, T: TelegramClient> Router<R, S, T> {
    pub fn new(client: T) -> Self {
        Self {
            api: Arc::new(API::new(client)),
            chat_handlers: vec![],
            chat_state: HashMap::new(),
        }
    }

    pub fn add_chat_handler(&mut self, h: impl Into<chat::Handler<R, S, T>>) {
        self.chat_handlers.push(h.into())
    }

    pub async fn handle_chat_op(&self, chat_id: i64, op: chat::Op) -> anyhow::Result<()> {
        match op {
            chat::Op::ReplyText(text) => {
                self.api
                    .send_message(&crate::SendMessageRequest {
                        chat_id,
                        text,
                        reply_to_message_id: None,
                    })
                    .await?;
            }
            chat::Op::ReplySticker(sticker) => {
                self.api
                    .send_sticker(&crate::SendStickerRequest::new(chat_id, sticker))
                    .await?;
            }
            chat::Op::None => {}
        }

        Ok(())
    }

    pub async fn start(&mut self) {
        let mut last_update_id = 0;

        loop {
            debug!("last_update_id = {}", last_update_id);
            let updates = self
                .api
                .get_updates(
                    &GetUpdatesRequest::new()
                        .with_timeout(60)
                        .with_offset(last_update_id + 1),
                )
                .await
                .unwrap();

            for update in updates {
                let mut message_event = None;

                if let Some(message) = update.message.clone() {
                    message_event = Some(chat::MessageEvent::New(message));
                } else if let Some(message) = update.edited_message.clone() {
                    message_event = Some(chat::MessageEvent::Edited(message));
                }

                last_update_id = max(last_update_id, update.update_id);

                match &message_event {
                    Some(chat::MessageEvent::New(message))
                    | Some(chat::MessageEvent::Edited(message)) => {
                        debug!("New message: {:?}", message);
                        let chat_id = message.chat.id;

                        for handler in &self.chat_handlers {
                            let state = self
                                .chat_state
                                .entry(chat_id)
                                .or_insert(handler.state.clone());

                            let reply = (handler.f)(
                                chat::Event {
                                    api: Arc::clone(&self.api),
                                    message: message_event.clone().unwrap(),
                                },
                                state.clone(),
                            )
                            .await
                            .unwrap();

                            match reply.into() {
                                chat::Action::Next(chat::Op::None) => {}
                                chat::Action::Done(chat::Op::None) => {
                                    break;
                                }
                                chat::Action::Next(op) => {
                                    self.handle_chat_op(chat_id, op).await.unwrap();
                                }
                                chat::Action::Done(op) => {
                                    self.handle_chat_op(chat_id, op).await.unwrap();
                                    break;
                                }
                            }
                        }
                    }
                    None => {
                        warn!("ChatHandler cannot handle update: {:?}", update)
                    }
                }
            }
        }
    }
}
