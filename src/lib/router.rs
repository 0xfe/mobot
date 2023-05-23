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

use futures::{future::BoxFuture, Future};
use tokio::sync::{mpsc, Notify, RwLock};

use crate::{
    api::{self, GetUpdatesRequest, SendMessageRequest, SendStickerRequest, Update},
    chat::{self, MessageEvent},
    handlers::query,
    Client, API,
};

type Arw<T> = Arc<RwLock<T>>;
type ErrorHandler =
    Box<dyn Fn(Arc<API>, i64, anyhow::Error) -> BoxFuture<'static, ()> + Send + Sync>;

/// `Matcher` is used to match a message against a route. It is used to determine
/// which handler should be called for a given message.
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Matcher {
    /// Match any message
    Any,

    /// Match messages that are exactly equivelant
    Exact(String),

    /// Match messages that start with the given string
    Prefix(String),

    /// Match messages using the given regex
    Regex(String),

    /// Handle bot commands (messages that start with "/")
    BotCommand(String),
}

/// `Route` is used to determine which handler should be called for a given message or query.
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Route {
    /// Handle any event
    Any(Matcher),

    /// Handle new private chat messages
    NewMessage(Matcher),

    /// Handle edited private chat messages
    EditedMessage(Matcher),

    /// Handle channel posts
    ChannelPost(Matcher),

    /// Handle edited channel posts
    EditedChannelPost(Matcher),

    /// Handle callback queries from inline keyboards
    CallbackQuery(Matcher),
}

pub struct Router<S: Clone> {
    api: Arc<API>,

    error_handler: Arc<ErrorHandler>,

    /// TODO: locks are too fine grained, break it up
    chat_handlers: Arw<HashMap<Route, Vec<chat::Handler<S>>>>,
    chat_state: Arw<HashMap<i64, chat::State<S>>>,
    query_handlers: Arw<Vec<query::Handler<S>>>,
    user_state: Arw<HashMap<i64, query::State<S>>>,

    /// Telegram getUpdates HTTP poll timeout
    timeout_s: i64,

    /// Shutdown notifier
    shutdown: Arc<Notify>,
    shutdown_tx: Arc<mpsc::Sender<()>>,
    shutdown_rx: mpsc::Receiver<()>,
}

async fn default_error_handler(api: Arc<API>, chat_id: i64, err: anyhow::Error) {
    error!("Error: {}", err);
    let result = api
        .send_message(&SendMessageRequest {
            chat_id,
            text: format!("Handler error: {}", err),
            ..Default::default()
        })
        .await;

    if let Err(err) = result {
        error!("Error in default error handler: {}", err);
    }
}

impl<S: Clone + Send + Sync + 'static> Router<S> {
    /// Create a new router with the given client.
    pub fn new(client: Client) -> Self {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        Self {
            api: Arc::new(API::new(client)),
            error_handler: Arc::new(Box::new(move |a, b, c| {
                Box::pin(default_error_handler(a, b, c))
            })),
            chat_handlers: Arc::new(RwLock::new(HashMap::new())),
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

    pub fn with_error_handler<Func, Fut>(mut self, func: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(Arc<API>, i64, anyhow::Error) -> Fut,
        Fut: Send + 'static + Future<Output = ()>,
    {
        self.error_handler = Arc::new(Box::new(move |a, b, c| Box::pin(func(a, b, c))));
        self
    }

    /// Add a handler for all messages in a chat. The handler is called with current
    /// state of the chat ID.
    pub async fn add_chat_handler(&mut self, h: impl Into<chat::Handler<S>>) {
        self.chat_handlers
            .write()
            .await
            .entry(Route::Any(Matcher::Any))
            .or_default()
            .push(h.into())
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
                debug!("Received update: {:#?}", update);
                last_update_id = max(last_update_id, update.update_id);

                let chat_update = update.clone();
                let chat_handlers = Arc::clone(&self.chat_handlers);
                let error_handler = Arc::clone(&self.error_handler);
                let chat_state = Arc::clone(&self.chat_state);
                let api = Arc::clone(&self.api);
                tokio::spawn(async move {
                    if let Err(err) = Self::handle_chat_update(
                        api,
                        chat_state,
                        chat_handlers,
                        error_handler,
                        chat_update,
                    )
                    .await
                    {
                        error!("Error handling chat update: {}", err);
                    }
                });

                let query_handlers = Arc::clone(&self.query_handlers);
                let error_handler = Arc::clone(&self.error_handler);
                let user_state = Arc::clone(&self.user_state);
                let api = Arc::clone(&self.api);
                tokio::spawn(async move {
                    if let Err(err) = Self::handle_query_update(
                        api,
                        user_state,
                        query_handlers,
                        error_handler,
                        update,
                    )
                    .await
                    {
                        error!("Error handling query update: {}", err);
                    }
                });
            }
        }

        self.shutdown.notify_waiters();
    }

    async fn handle_chat_update(
        api: Arc<API>,
        chat_state: Arc<RwLock<HashMap<i64, chat::State<S>>>>,
        chat_handlers: Arc<RwLock<HashMap<Route, Vec<chat::Handler<S>>>>>,
        error_handler: Arc<ErrorHandler>,
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
        } else if let Some(ref m) = update.channel_post {
            debug!("Channel post: {:#?}", m);
            chat_id = m.chat.id;
            message_event = MessageEvent::Post(m.clone());
        } else if let Some(ref m) = update.edited_channel_post {
            debug!("Edited channel post: {:#?}", m);
            chat_id = m.chat.id;
            message_event = MessageEvent::EditedPost(m.clone());
        } else if let Some(ref q) = update.callback_query {
            debug!("Callback query: {:#?}", q);
            chat_id = q.message.as_ref().map(|m| m.chat.id).unwrap_or(0);
            message_event = MessageEvent::Callback(q.clone());
        } else {
            return Ok(());
        }

        for handler in chat_handlers
            .read()
            .await
            .get(&Route::Any(Matcher::Any))
            .unwrap_or(&Vec::new())
            .iter()
        {
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
            .await;

            if let Err(err) = reply {
                error_handler(Arc::clone(&api), chat_id, err).await;
                return Ok(());
            }

            match reply.unwrap() {
                chat::Action::Next => {}
                chat::Action::Done => {
                    break;
                }
                chat::Action::ReplyText(text) => {
                    api.send_message(&SendMessageRequest {
                        chat_id,
                        text,
                        ..Default::default()
                    })
                    .await?;
                }
                chat::Action::ReplyMarkdown(text) => {
                    api.send_message(&SendMessageRequest {
                        chat_id,
                        text,
                        reply_to_message_id: None,
                        parse_mode: Some(api::ParseMode::MarkdownV2),
                        ..Default::default()
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
        error_handler: Arc<ErrorHandler>,
        update: Update,
    ) -> anyhow::Result<()> {
        let Some(ref query) = update.inline_query else {
            return Ok(());
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
            .await;

            if let Err(err) = reply {
                error_handler(Arc::clone(&api), query.from.id, err).await;
                return Ok(());
            }

            match reply.unwrap() {
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
