/// `Router` is the main entry point to the bot. It is used to register handlers
/// for different types of events, and keeps track of the state of the bot,
/// passing it to the right handler.
///
/// Chat Handlers are called for every message that is sent to the bot that is part
/// of a chat session. The router keeps track of the state of each chat session,
/// and passes the relevant state for the current Chat ID to the handler.
///
/// User handlers are called for every message that is sent to the bot from any specific
/// user.
use std::{cmp::max, collections::HashMap, sync::Arc, time::Duration};

use futures::{future::BoxFuture, Future};
use tokio::sync::{mpsc, Notify, RwLock};

use crate::{
    api::{self, GetUpdatesRequest, SendMessageRequest, SendStickerRequest, API},
    handler::{BotHandler, BotState},
    Action, Client, Event, State, Update,
};

use anyhow::anyhow;

type Arw<T> = Arc<RwLock<T>>;
type HandlerMap<S> = HashMap<Route, Vec<(Matcher, Box<dyn BotHandler<S>>)>>;
type ErrorHandler<S> =
    Box<dyn Fn(Arc<API>, i64, State<S>, anyhow::Error) -> BoxFuture<'static, ()> + Send + Sync>;

/// `Matcher` is used to match a message against a route. It is used to determine
/// which handler should be called for a given message.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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

    /// Match messages that represent a photo
    Photo,

    /// Match messages that represent a general file
    Document,
}

impl Matcher {
    pub fn match_str(&self, s: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(m) => s == m,
            Self::Prefix(m) => s.starts_with(m),
            Self::Regex(m) => regex::Regex::new(m).unwrap().is_match(s),
            Self::BotCommand(m) => s.starts_with(&format!("/{}", m)),
            Self::Document | Self::Photo => false,
        }
    }
}

impl From<Route> for Matcher {
    fn from(r: Route) -> Self {
        match r {
            Route::Default => Matcher::Any,
            Route::Any(matcher) => matcher,
            Route::Message(matcher) => matcher,
            Route::EditedMessage(matcher) => matcher,
            Route::ChannelPost(matcher) => matcher,
            Route::EditedChannelPost(matcher) => matcher,
            Route::CallbackQuery(matcher) => matcher,
            Route::InlineQuery(matcher) => matcher,
        }
    }
}

/// `Route` is used to determine which handler should be called for a given message or query.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Route {
    /// Handle any event (alias for Any(Any))
    Default,

    /// Handle any event
    Any(Matcher),

    /// Handle new private chat messages
    Message(Matcher),

    /// Handle edited private chat messages
    EditedMessage(Matcher),

    /// Handle channel posts
    ChannelPost(Matcher),

    /// Handle edited channel posts
    EditedChannelPost(Matcher),

    /// Handle callback queries from inline keyboards
    CallbackQuery(Matcher),

    /// Handle inline queries
    InlineQuery(Matcher),
}

fn get_update_parts(update: &api::Update) -> anyhow::Result<(i64, Route)> {
    if let Some(ref m) = update.message {
        debug!("New message: {:#?}", m);
        Ok((m.chat.id, Route::Message(Matcher::Any)))
    } else if let Some(ref m) = update.edited_message {
        debug!("Edited message: {:#?}", m);
        Ok((m.chat.id, Route::EditedMessage(Matcher::Any)))
    } else if let Some(ref m) = update.channel_post {
        debug!("Channel post: {:#?}", m);
        Ok((m.chat.id, Route::ChannelPost(Matcher::Any)))
    } else if let Some(ref m) = update.edited_channel_post {
        debug!("Edited channel post: {:#?}", m);
        Ok((m.chat.id, Route::EditedChannelPost(Matcher::Any)))
    } else if let Some(ref q) = update.callback_query {
        debug!("Callback query: {:#?}", q);
        Ok((
            q.message.as_ref().map(|m| m.chat.id).unwrap_or(0),
            Route::CallbackQuery(Matcher::Any),
        ))
    } else if let Some(ref q) = update.inline_query {
        debug!("Inline query: {:#?}", q);
        Ok((q.from.id, Route::InlineQuery(Matcher::Any)))
    } else {
        anyhow::bail!("Unknown update type")
    }
}

impl Route {
    pub fn any(r: &Route) -> Self {
        match r {
            Self::Default => Self::Any(Matcher::Any),
            Self::Any(_) => Self::Any(Matcher::Any),
            Self::Message(_) => Self::Message(Matcher::Any),
            Self::EditedMessage(_) => Self::EditedMessage(Matcher::Any),
            Self::ChannelPost(_) => Self::ChannelPost(Matcher::Any),
            Self::EditedChannelPost(_) => Self::EditedChannelPost(Matcher::Any),
            Self::CallbackQuery(_) => Self::CallbackQuery(Matcher::Any),
            Self::InlineQuery(_) => Self::InlineQuery(Matcher::Any),
        }
    }

    pub fn with(&self, matcher: &Matcher) -> Self {
        match self {
            Self::Default => Self::Any(matcher.clone()),
            Self::Any(_) => Self::Any(matcher.clone()),
            Self::Message(_) => Self::Message(matcher.clone()),
            Self::EditedMessage(_) => Self::EditedMessage(matcher.clone()),
            Self::ChannelPost(_) => Self::ChannelPost(matcher.clone()),
            Self::EditedChannelPost(_) => Self::EditedChannelPost(matcher.clone()),
            Self::CallbackQuery(_) => Self::CallbackQuery(matcher.clone()),
            Self::InlineQuery(_) => Self::InlineQuery(matcher.clone()),
        }
    }

    pub fn match_update(&self, update: &api::Update) -> bool {
        match self {
            Self::Message(m) => match m {
                Matcher::Photo => update
                    .message
                    .as_ref()
                    .and_then(|m| m.photo.as_ref())
                    .is_some(),
                Matcher::Document => update
                    .message
                    .as_ref()
                    .and_then(|m| m.document.as_ref())
                    .is_some(),
                _ => update
                    .message
                    .as_ref()
                    .and_then(|m| m.text.as_ref())
                    .map_or(false, |t| m.match_str(t)),
            },
            Self::EditedMessage(m) => update
                .edited_message
                .as_ref()
                .and_then(|m| m.text.as_ref())
                .map_or(false, |t| m.match_str(t)),
            Self::ChannelPost(m) => update
                .channel_post
                .as_ref()
                .and_then(|m| m.text.as_ref())
                .map_or(false, |t| m.match_str(t)),
            Self::EditedChannelPost(m) => update
                .edited_channel_post
                .as_ref()
                .and_then(|m| m.text.as_ref())
                .map_or(false, |t| m.match_str(t)),
            Self::CallbackQuery(m) => update
                .callback_query
                .as_ref()
                .and_then(|m| m.data.as_ref())
                .map_or(false, |t| m.match_str(t)),
            Self::InlineQuery(m) => update
                .inline_query
                .as_ref()
                .map_or(false, |t| m.match_str(&t.query)),
            Self::Any(matcher) => {
                let mut matched = false;
                if let Some(ref m) = update.message {
                    matched |= m.text.as_ref().map_or(false, |t| matcher.match_str(t));
                }
                if let Some(ref m) = update.edited_message {
                    matched |= m.text.as_ref().map_or(false, |t| matcher.match_str(t));
                }
                if let Some(ref m) = update.channel_post {
                    matched |= m.text.as_ref().map_or(false, |t| matcher.match_str(t));
                }
                if let Some(ref m) = update.edited_channel_post {
                    matched |= m.text.as_ref().map_or(false, |t| matcher.match_str(t));
                }
                if let Some(ref q) = update.callback_query {
                    matched |= q.data.as_ref().map_or(false, |t| matcher.match_str(t));
                }
                if let Some(ref q) = update.inline_query {
                    matched |= matcher.match_str(&q.query);
                }
                matched
            }
            Self::Default => true,
        }
    }
}

pub struct Router<S: BotState> {
    pub api: Arc<API>,
    state: Option<Arc<RwLock<S>>>,

    error_handler: Arc<ErrorHandler<S>>,

    /// TODO: locks are too fine grained, break it up
    init_handlers: Option<HandlerMap<S>>,
    handlers: Arw<HandlerMap<S>>,
    handler_state: Arw<HashMap<i64, State<S>>>,

    /// Telegram getUpdates HTTP poll timeout
    timeout_s: i64,

    /// Shutdown notifier
    shutdown: Arc<Notify>,
    shutdown_tx: Arc<mpsc::Sender<()>>,
    shutdown_rx: mpsc::Receiver<()>,
}

async fn default_error_handler<S: BotState>(
    api: Arc<API>,
    chat_id: i64,
    _: State<S>,
    err: anyhow::Error,
) {
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

impl<S: BotState> Router<S> {
    /// Create a new router with the given client.
    pub fn new(client: Client) -> Self {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        Self {
            api: Arc::new(API::new(client)),
            state: None,
            error_handler: Arc::new(Box::new(move |a, b, c, d| {
                Box::pin(default_error_handler(a, b, c, d))
            })),
            init_handlers: Some(HashMap::new()),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            handler_state: Arc::new(RwLock::new(HashMap::new())),
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

    pub fn with_state(mut self, state: S) -> Self {
        self.state = Some(Arc::new(RwLock::new(state)));
        self
    }

    pub fn with_error_handler<Func, Fut>(mut self, func: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(Arc<API>, i64, State<S>, anyhow::Error) -> Fut,
        Fut: Send + 'static + Future<Output = ()>,
    {
        self.error_handler = Arc::new(Box::new(move |a, b, c, d| Box::pin(func(a, b, c, d))));
        self
    }

    /// Add a handler for messages matching a route in a chat. The handler is called with current
    /// state of the chat ID or the user ID, depending on the update.
    pub fn add_route(&mut self, r: Route, h: impl Into<Box<dyn BotHandler<S>>>) -> &mut Self {
        let mut h: Box<dyn BotHandler<S>> = h.into();

        if let Some(state) = &self.state {
            h.set_state(Arc::clone(state));
        }

        // Note that Route::Default gets converted to Route::Any(Matcher::Any)
        self.init_handlers
            .as_mut()
            .expect("Can't call add_chat_route after start()")
            .entry(Route::any(&r))
            .or_default()
            .push((r.into(), h));

        self
    }

    pub fn shutdown(&self) -> (Arc<Notify>, Arc<mpsc::Sender<()>>) {
        (Arc::clone(&self.shutdown), Arc::clone(&self.shutdown_tx))
    }

    /// Start the router. This will block forever.
    pub async fn start(&mut self) {
        let mut last_update_id = 0;

        // Move chat handlers from init_chat_handlers to chat_handlers so it can be passed on
        // to other tasks.
        self.handlers = Arc::new(RwLock::new(self.init_handlers.take().unwrap()));

        loop {
            if self.shutdown_rx.try_recv().is_ok() {
                info!("Received shutdown signal");
                break;
            }

            debug!(
                "Polling /getUpdates with last_update_id = {} timeout = {}s",
                last_update_id, self.timeout_s
            );

            let updates = match self
                .api
                .get_updates(
                    &GetUpdatesRequest::new()
                        .with_timeout(self.timeout_s)
                        .with_offset(last_update_id + 1),
                )
                .await
            {
                Ok(updates) => updates,
                Err(err) => {
                    error!("Error polling /getUpdates: {}", err);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            for update in updates {
                debug!("Received update: {:#?}", update);
                last_update_id = max(last_update_id, update.update_id);

                let chat_update = update.clone();
                let handlers = Arc::clone(&self.handlers);
                let error_handler = Arc::clone(&self.error_handler);
                let handler_state = Arc::clone(&self.handler_state);
                let api = Arc::clone(&self.api);
                tokio::spawn(async move {
                    if let Err(err) = Self::handle_chat_update(
                        api,
                        handler_state,
                        handlers,
                        error_handler,
                        chat_update,
                    )
                    .await
                    {
                        error!("Error handling chat update: {}", err);
                    }
                });
            }
        }

        self.shutdown.notify_waiters();
    }

    async fn handle_chat_update(
        api: Arc<API>,
        handler_state: Arc<RwLock<HashMap<i64, State<S>>>>,
        handlers: Arw<HandlerMap<S>>,
        error_handler: Arc<ErrorHandler<S>>,
        update: api::Update,
    ) -> anyhow::Result<()> {
        let (chat_id, route) = get_update_parts(&update)?;
        let message_event: Update = update.clone().into();

        let mut handler_groups = vec![];
        let h = handlers.read().await;

        // Check to see if there's a handler stack for this message's route.
        if let Some(handlers) = h.get(&route) {
            handler_groups.push(handlers);
        } else {
            // Check to see if there's a default handler.
            if let Some(handlers) = h.get(&Route::Any(Matcher::Any)) {
                handler_groups.push(handlers);
            }
        }

        if handler_groups.is_empty() {
            // No default handler installed, so we can't do anything with this message. Call
            // the error handler.
            error_handler(
                Arc::clone(&api),
                chat_id,
                State::default(),
                anyhow!(format!("No handlers installed for route: #{:?}", route)),
            )
            .await;
        }

        // Go through each handler in the stack and see if it matches the update.
        'top: for handler_group in handler_groups {
            for matcher_handler in handler_group {
                let (matcher, handler) = matcher_handler;
                if !route.with(matcher).match_update(&update) {
                    // Route doesn't match, so skip this handler.
                    continue;
                }

                // If we don't have a state for this chat, create one by cloning
                // the initial state stored in the handler.
                let state = {
                    let mut state = handler_state.write().await;
                    state
                        .entry(chat_id)
                        .or_insert(State::from(handler.get_state()).await)
                        .clone()
                };

                // Run the handler
                let reply = handler
                    .run(
                        Event::new(Arc::clone(&api), message_event.clone()),
                        state.clone(),
                    )
                    .await;

                // Handler failed, run the default error handler
                if let Err(err) = reply {
                    error_handler(Arc::clone(&api), chat_id, state, err).await;
                    return Ok(());
                }

                match reply.unwrap() {
                    // Handler returned Next, run the next handler in the stack.
                    Action::Next => {}

                    // Handler returned Done, stop running handlers.
                    Action::Done => {
                        break 'top;
                    }

                    // Handler returned Reply, send the message to the chat, and stop running handlers.
                    Action::ReplyText(text) => {
                        api.send_message(&SendMessageRequest {
                            chat_id,
                            text,
                            ..Default::default()
                        })
                        .await?;
                        break 'top;
                    }

                    // Handler returned ReplyMarkdown, send the MarkDown message to the chat, and
                    // stop running handlers.
                    Action::ReplyMarkdown(text) => {
                        api.send_message(&SendMessageRequest {
                            chat_id,
                            text,
                            parse_mode: Some(api::ParseMode::MarkdownV2),
                            ..Default::default()
                        })
                        .await?;
                        break 'top;
                    }

                    // Handler returned ReplySticker, send the sticker to the chat, and stop running
                    // handlers.
                    Action::ReplySticker(sticker) => {
                        api.send_sticker(&SendStickerRequest::new(chat_id, sticker))
                            .await?;
                        break 'top;
                    }
                }
            }
        }
        Ok(())
    }
}
