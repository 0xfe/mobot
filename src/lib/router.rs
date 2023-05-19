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
use tokio::sync::{mpsc, Notify, RwLock};

use crate::{
    api::{self, GetUpdatesRequest, SendMessageRequest, SendStickerRequest, Update},
    chat::{self, MessageEvent},
    handlers::query,
    Client, API,
};

type Arw<T> = Arc<RwLock<T>>;

pub struct Router<S: Clone> {
    api: Arc<API>,

    /// TODO: locks are too fine grained, break it up
    chat_handlers: Arw<Vec<chat::Handler<S>>>,
    chat_state: Arw<HashMap<i64, chat::State<S>>>,
    query_handlers: Arw<Vec<query::Handler<S>>>,
    user_state: Arw<HashMap<i64, query::State<S>>>,

    /// HTTP poll timeout
    timeout_s: i64,

    /// Shutdown notifier
    shutdown: Arc<Notify>,
    shutdown_tx: Arc<mpsc::Sender<()>>,
    shutdown_rx: mpsc::Receiver<()>,
}

impl<S: Clone + Send + Sync + 'static> Router<S> {
    /// Create a new router with the given client.
    pub fn new(client: Client) -> Self {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        Self {
            api: Arc::new(API::new(client)),
            chat_handlers: Arc::new(RwLock::new(vec![])),
            query_handlers: Arc::new(RwLock::new(vec![])),
            chat_state: Arc::new(RwLock::new(HashMap::new())),
            user_state: Arc::new(RwLock::new(HashMap::new())),
            timeout_s: 60,
            shutdown: Arc::new(Notify::new()),
            shutdown_tx: Arc::new(shutdown_tx),
            shutdown_rx,
        }
    }

    pub fn with_poll_timeout_s(mut self, timeout_s: i64) -> Self {
        self.timeout_s = timeout_s;
        self
    }

    /// Add a handler for all messages in a chat. The handler is called with current
    /// state of the chat ID.
    pub async fn add_chat_handler(&mut self, h: impl Into<chat::Handler<S>>) {
        self.chat_handlers.write().await.push(h.into());
    }

    /// Add a handler for all queries. The handler is called with current state
    /// of the user ID.
    pub async fn add_query_handler(&mut self, h: impl Into<query::Handler<S>>) {
        self.query_handlers.write().await.push(h.into())
    }

    pub fn shutdown(&self) -> (Arc<Notify>, Arc<mpsc::Sender<()>>) {
        (Arc::clone(&self.shutdown), Arc::clone(&self.shutdown_tx))
    }

    /// Start the router. This will block forever.
    pub async fn start(&mut self) {
        let mut last_update_id = 0;

        loop {
            if self.shutdown_rx.try_recv().is_ok() {
                info!("Received shutdown signal");
                break;
            }

            debug!(
                "Polling /getUpdates with last_update_id = {} timeout = {}s",
                last_update_id, self.timeout_s
            );
            let updates = self
                .api
                .get_updates(
                    &GetUpdatesRequest::new()
                        .with_timeout(self.timeout_s)
                        .with_offset(last_update_id + 1),
                )
                .await
                .unwrap();

            for update in updates {
                last_update_id = max(last_update_id, update.update_id);

                let chat_update = update.clone();
                let chat_handlers = Arc::clone(&self.chat_handlers);
                let chat_state = Arc::clone(&self.chat_state);
                let api = Arc::clone(&self.api);
                tokio::spawn(async move {
                    _ = Self::handle_chat_update(api, chat_state, chat_handlers, chat_update).await;
                });

                let query_handlers = Arc::clone(&self.query_handlers);
                let user_state = Arc::clone(&self.user_state);
                let api = Arc::clone(&self.api);
                tokio::spawn(async move {
                    _ = Self::handle_query_update(api, user_state, query_handlers, update).await;
                });
            }
        }

        self.shutdown.notify_waiters();
    }

    async fn handle_chat_update(
        api: Arc<API>,
        chat_state: Arc<RwLock<HashMap<i64, chat::State<S>>>>,
        chat_handlers: Arc<RwLock<Vec<chat::Handler<S>>>>,
        update: Update,
    ) -> anyhow::Result<()> {
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

        for handler in chat_handlers.read().await.iter() {
            // If we don't have a state for this chat, create one by cloning
            // the initial state stored in the handler.
            let state = {
                let mut state = chat_state.write().await;
                state
                    .entry(chat_id)
                    .or_insert(chat::State::from(&handler.state).await)
                    .clone()
            };

            let reply = (handler.f)(
                chat::Event {
                    api: Arc::clone(&api),
                    message: message_event.clone(),
                },
                state,
            )
            .await?;

            match reply {
                chat::Action::Next => {}
                chat::Action::Done => {
                    break;
                }
                chat::Action::ReplyText(text) => {
                    api.send_message(&SendMessageRequest {
                        chat_id,
                        text,
                        reply_to_message_id: None,
                        parse_mode: None,
                    })
                    .await?;
                }
                chat::Action::ReplyMarkdown(text) => {
                    api.send_message(&SendMessageRequest {
                        chat_id,
                        text,
                        reply_to_message_id: None,
                        parse_mode: Some("MarkdownV2".into()),
                    })
                    .await?;
                }
                chat::Action::ReplySticker(sticker) => {
                    api.send_sticker(&SendStickerRequest::new(chat_id, sticker))
                        .await?;
                }
            }
        }
        Ok(())
    }

    async fn handle_query_update(
        api: Arc<API>,
        user_state: Arc<RwLock<HashMap<i64, query::State<S>>>>,
        query_handlers: Arc<RwLock<Vec<query::Handler<S>>>>,
        update: Update,
    ) -> anyhow::Result<()> {
        let Some(ref query) = update.inline_query else {
            bail!("Update is not a query");
        };

        for handler in query_handlers.read().await.iter() {
            let state = {
                user_state
                    .write()
                    .await
                    .entry(query.from.id)
                    .or_insert(query::State::from(&handler.state).await)
                    .clone()
            };

            let reply = (handler.f)(
                query::Event {
                    api: Arc::clone(&api),
                    query: query.clone(),
                },
                state,
            )
            .await
            .unwrap();

            match reply {
                query::Action::Next => {}
                query::Action::Done => {
                    break;
                }
                query::Action::ReplyText(title, text) => {
                    api.answer_inline_query(
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
