use std::{cmp::max, collections::HashMap, sync::Arc};

use crate::{
    Action, ChatAction, ChatEvent, ChatHandler, GetUpdatesRequest, MessageEvent, TelegramClient,
    UpdateEvent, API,
};

// Handler routing:
//  - by chat ID
//  - by user
//  - by message type
//  - by message text regex
//
// create filtering functions for each of these, and then compose them together

pub struct Router<R, S, T>
where
    R: Into<Action<ChatAction>>,
    T: TelegramClient,
{
    api: Arc<API<T>>,
    chat_handlers: Vec<ChatHandler<R, S, T>>,
    chat_state: HashMap<i64, S>,
}

impl<R: Into<Action<ChatAction>>, S: Clone, T: TelegramClient> Router<R, S, T> {
    pub fn new(client: T) -> Self {
        Self {
            api: Arc::new(API::new(client)),
            chat_handlers: vec![],
            chat_state: HashMap::new(),
        }
    }

    pub fn add_chat_handler(&mut self, h: ChatHandler<R, S, T>) {
        self.chat_handlers.push(h)
    }

    pub async fn handle_action(&self, chat_id: i64, action: ChatAction) -> anyhow::Result<()> {
        match action {
            ChatAction::ReplyText(text) => {
                self.api
                    .send_message(&crate::SendMessageRequest {
                        chat_id,
                        text,
                        reply_to_message_id: None,
                    })
                    .await?;
            }
            ChatAction::ReplySticker(sticker) => {
                self.api
                    .send_sticker(&crate::SendStickerRequest::new(chat_id, sticker))
                    .await?;
            }
            ChatAction::None => {}
        }

        Ok(())
    }

    pub async fn start(&mut self) {
        let mut last_update_id = 0;

        loop {
            debug!("last_update_id = {}", last_update_id);
            let updates = self
                .api
                .get_update_events(
                    &GetUpdatesRequest::new()
                        .with_timeout(60)
                        .with_offset(last_update_id + 1),
                )
                .await
                .unwrap();

            for update in updates {
                match update {
                    UpdateEvent::NewMessage(id, message) => {
                        last_update_id = max(last_update_id, id);
                        let chat_id = message.chat.id;

                        for handler in &self.chat_handlers {
                            let state = self
                                .chat_state
                                .entry(chat_id)
                                .or_insert(handler.state.clone());

                            let reply = (handler.f)(
                                ChatEvent {
                                    api: Arc::clone(&self.api),
                                    message: MessageEvent::New(message.clone()),
                                },
                                state.clone(),
                            )
                            .await
                            .unwrap();

                            match reply.into() {
                                Action::Next(ChatAction::None) => {}
                                Action::Done(ChatAction::None) => {
                                    break;
                                }
                                Action::Next(action) => {
                                    self.handle_action(chat_id, action).await.unwrap();
                                }
                                Action::Done(action) => {
                                    self.handle_action(chat_id, action).await.unwrap();
                                    break;
                                }
                            }
                        }
                    }
                    _ => {
                        warn!("Unhandled update: {update:?}");
                    }
                }
            }
        }
    }
}
