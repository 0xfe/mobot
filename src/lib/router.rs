use std::{cmp::max, collections::HashMap, sync::Arc};

use anyhow::anyhow;

use crate::{
    chat::{self, MessageEvent},
    handlers::query,
    Client, GetUpdatesRequest, Update, API,
};

// Handler routing:
//  - by chat ID
//  - by user
//  - by message type
//  - by message text regex
//
// create filtering functions for each of these, and then compose them together

pub struct Router<S> {
    api: Arc<API>,
    chat_handlers: Vec<chat::Handler<S>>,
    query_handlers: Vec<query::Handler<S>>,
    chat_state: HashMap<i64, S>,
}

impl<S> Router<S>
where
    S: Clone + Default,
{
    pub fn new(client: Client) -> Self {
        Self {
            api: Arc::new(API::new(client)),
            chat_handlers: vec![],
            chat_state: HashMap::new(),
            query_handlers: vec![],
        }
    }

    pub fn add_chat_handler(&mut self, h: impl Into<chat::Handler<S>>) {
        self.chat_handlers.push(h.into())
    }

    pub fn add_query_handler(&mut self, h: impl Into<query::Handler<S>>) {
        self.query_handlers.push(h.into())
    }

    async fn handle_chat_update(&mut self, update: &Update) -> anyhow::Result<()> {
        let message_event;
        let chat_id;

        if let Some(ref m) = update.message {
            debug!("New message: {:#?}", m);
            chat_id = m.chat.id;
            message_event = MessageEvent::New(m.clone());
        } else if let Some(ref m) = update.edited_message {
            debug!("Edited message: {:#?}", m);
            chat_id = m.chat.id;
            message_event = MessageEvent::Edited(m.clone());
        } else {
            return Err(anyhow!("Update is not a message"));
        }

        for handler in &self.chat_handlers {
            let state = self
                .chat_state
                .entry(chat_id)
                .or_insert(handler.state.clone());

            let reply = (handler.f)(
                chat::Event {
                    api: Arc::clone(&self.api),
                    message: message_event.clone(),
                },
                state.clone(),
            )
            .await
            .unwrap();

            match reply {
                chat::Action::Next => {}
                chat::Action::Done => {
                    break;
                }
                chat::Action::ReplyText(text) => {
                    self.api
                        .send_message(&crate::SendMessageRequest {
                            chat_id,
                            text,
                            reply_to_message_id: None,
                        })
                        .await?;
                }
                chat::Action::ReplySticker(sticker) => {
                    self.api
                        .send_sticker(&crate::SendStickerRequest::new(chat_id, sticker))
                        .await?;
                }
            }
        }
        Ok(())
    }

    async fn handle_query_update(&mut self, update: &Update) -> anyhow::Result<()> {
        let Some(ref query) = update.inline_query else {
            return Err(anyhow!("Update is not a query"));
        };

        for handler in &self.query_handlers {
            let reply = (handler.f)(
                query::Event {
                    api: Arc::clone(&self.api),
                    query: query.clone(),
                },
                S::default(),
            )
            .await
            .unwrap();

            match reply {
                query::Action::Next => {}
                query::Action::Done => {
                    break;
                }
                query::Action::ReplyText(text) => {
                    self.api
                        .answer_inline_query_with_text(query.id.clone(), text)
                        .await?;
                }
            }
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
                debug!("New update: {:#?}", update);
                last_update_id = max(last_update_id, update.update_id);
                _ = self.handle_chat_update(&update).await;
                _ = self.handle_query_update(&update).await;
            }
        }
    }
}
