/// `Router` is the main entry point to the bot. It is used to register handlers
/// for different types of events, and keeps track of the state of the bot,
/// passing it to the right handler.
///
/// Currently, Routers support two types of hanlders:
///
/// - `chat::Handler` for chat events
/// - `query::Handler` for inline query events
///
/// Chat handlers are called for every message that is sent to the bot that is part
/// of a chat session. The router keeps track of the state of each chat session,
/// and passes the relevant state for the current Chat ID to the handler.
///
/// Query handlers are called for every inline query that is sent to the bot.
use std::{cmp::max, collections::HashMap, sync::Arc};

use anyhow::bail;

use crate::{
    api::{self, GetUpdatesRequest, SendMessageRequest, SendStickerRequest, Update},
    chat::{self, MessageEvent},
    handlers::query,
    Client, API,
};

pub struct Router<S> {
    api: Arc<API>,
    chat_handlers: Vec<chat::Handler<S>>,
    query_handlers: Vec<query::Handler<S>>,
    chat_state: HashMap<i64, S>,
    user_state: HashMap<i64, S>,
}

impl<S: Clone> Router<S> {
    /// Create a new router with the given client.
    pub fn new(client: Client) -> Self {
        Self {
            api: Arc::new(API::new(client)),
            chat_handlers: vec![],
            query_handlers: vec![],
            chat_state: HashMap::new(),
            user_state: HashMap::new(),
        }
    }

    /// Add a handler for all messages in a chat. The handler is called with current
    /// state of the chat ID.
    pub fn add_chat_handler(&mut self, h: impl Into<chat::Handler<S>>) {
        self.chat_handlers.push(h.into())
    }

    /// Add a handler for all queries. The handler is called with current state
    /// of the user ID.
    pub fn add_query_handler(&mut self, h: impl Into<query::Handler<S>>) {
        self.query_handlers.push(h.into())
    }

    /// Start the router. This will block forever.
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
                last_update_id = max(last_update_id, update.update_id);
                _ = self.handle_chat_update(&update).await;
                _ = self.handle_query_update(&update).await;
            }
        }
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
            bail!("Update is not a message");
        }

        for handler in &self.chat_handlers {
            // If we don't have a state for this chat, create one by cloning
            // the initial state stored in the handler.
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
            .await?;

            match reply {
                chat::Action::Next => {}
                chat::Action::Done => {
                    break;
                }
                chat::Action::ReplyText(text) => {
                    self.api
                        .send_message(&SendMessageRequest {
                            chat_id,
                            text,
                            reply_to_message_id: None,
                            parse_mode: None,
                        })
                        .await?;
                }
                chat::Action::ReplyMarkdown(text) => {
                    self.api
                        .send_message(&SendMessageRequest {
                            chat_id,
                            text,
                            reply_to_message_id: None,
                            parse_mode: Some("MarkdownV2".into()),
                        })
                        .await?;
                }
                chat::Action::ReplySticker(sticker) => {
                    self.api
                        .send_sticker(&SendStickerRequest::new(chat_id, sticker))
                        .await?;
                }
            }
        }
        Ok(())
    }

    async fn handle_query_update(&mut self, update: &Update) -> anyhow::Result<()> {
        let Some(ref query) = update.inline_query else {
            bail!("Update is not a query");
        };

        for handler in &self.query_handlers {
            let state = self
                .user_state
                .entry(query.from.id)
                .or_insert(handler.state.clone());

            let reply = (handler.f)(
                query::Event {
                    api: Arc::clone(&self.api),
                    query: query.clone(),
                },
                state.clone(),
            )
            .await
            .unwrap();

            match reply {
                query::Action::Next => {}
                query::Action::Done => {
                    break;
                }
                query::Action::ReplyText(title, text) => {
                    self.api
                        .answer_inline_query(
                            &api::AnswerInlineQuery::new(query.id.clone())
                                .with_article_text(title, text),
                        )
                        .await?;
                }
            }
        }
        Ok(())
    }
}
