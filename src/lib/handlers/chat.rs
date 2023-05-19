use std::sync::Arc;

use futures::{future::BoxFuture, Future};
use tokio::sync::RwLock;

use crate::{api::Message, API};

#[derive(Clone, Default)]
pub struct State<T: Clone> {
    state: Arc<RwLock<T>>,
}

impl<T: Clone> State<T> {
    pub async fn from(&self) -> Self {
        Self {
            state: Arc::new(RwLock::new((*self.state.read().await).clone())),
        }
    }

    pub fn get(&self) -> &Arc<RwLock<T>> {
        &self.state
    }
}

/// `Event` represents an event sent to a chat handler.
#[derive(Clone)]
pub struct Event {
    pub api: Arc<API>,
    pub message: MessageEvent,
}

/// `MessageEvent` represents a new or edited message.
#[derive(Debug, Clone)]
pub enum MessageEvent {
    New(Message),
    Edited(Message),
}

/// `Action` represents an action to take after handling a chat event.
pub enum Action {
    /// Continue to the next handler.
    Next,

    /// Stop handling events.
    Done,

    /// Reply to the message with the given text and continue
    /// to the next handler.
    ReplyText(String),

    /// Same as ReplyText, but with MarkdownV2 formatting. Make
    /// sure to escape any user input!
    ReplyMarkdown(String),

    /// Reply to the message with the given sticker and continue
    /// to the next handler.
    ReplySticker(String),
}

/// A handler for a specific chat ID. This is a wrapper around an async function
/// that takes a `ChatEvent` and returns a `ChatAction`.
pub struct Handler<S: Clone> {
    /// Wraps the async handler function.
    #[allow(clippy::type_complexity)]
    pub f: Box<
        dyn Fn(Event, State<S>) -> BoxFuture<'static, Result<Action, anyhow::Error>> + Send + Sync,
    >,

    /// State related to this Chat ID
    pub state: State<S>,
}

impl<S: Clone> Handler<S>
where
    S: Default,
{
    pub fn new<Func, Fut>(func: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(Event, State<S>) -> Fut,
        Fut: Send + 'static + Future<Output = Result<Action, anyhow::Error>>,
    {
        Self {
            f: Box::new(move |a, b| Box::pin(func(a, b))),
            state: State {
                state: Arc::new(tokio::sync::RwLock::new(S::default())),
            },
        }
    }

    pub fn with_state(self, state: S) -> Self {
        Self {
            f: self.f,
            state: State {
                state: Arc::new(tokio::sync::RwLock::new(state)),
            },
        }
    }
}

impl<S, Func, Fut> From<Func> for Handler<S>
where
    S: Default + Clone,
    Func: Send + Sync + 'static + Fn(Event, State<S>) -> Fut,
    Fut: Send + 'static + Future<Output = Result<Action, anyhow::Error>>,
{
    fn from(func: Func) -> Self {
        Self::new(func)
    }
}

/// This handler logs every message received.
pub async fn log_handler<S>(e: Event, _: S) -> Result<Action, anyhow::Error> {
    match e.message {
        MessageEvent::New(message) | MessageEvent::Edited(message) => {
            let chat_id = message.chat.id;
            let from = message.from.unwrap();
            let text = message.text.unwrap_or_default();

            info!("({}) Message from {}: {}", chat_id, from.first_name, text);

            Ok(Action::Next)
        }
    }
}
