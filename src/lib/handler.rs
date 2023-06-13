use std::sync::Arc;

use async_trait::async_trait;
use futures::{future::BoxFuture, Future};
use tokio::sync::RwLock;

use crate::Event;

pub trait BotState: Default + Clone + Send + Sync + 'static {}
impl BotState for () {}

#[derive(Clone, Default)]
pub struct State<T: BotState> {
    state: Arc<RwLock<T>>,
}

impl<T: BotState> State<T> {
    pub async fn from(&self) -> Self {
        Self {
            state: Arc::new(RwLock::new((*self.state.read().await).clone())),
        }
    }

    pub fn get(&self) -> &Arc<RwLock<T>> {
        &self.state
    }
}

/// `Action` represents an action to take after handling a chat event.
#[derive(Debug, Clone)]
pub enum Action {
    /// Continue to the next handler.
    Next,

    /// Stop handling events.
    Done,

    /// Reply to the message with the given text and stop handling events. This
    /// is equivalent to `e.send_text(...)` followed by `Ok(Action::Done)`.
    ReplyText(String),

    /// Same as ReplyText, but with MarkdownV2 formatting. Make
    /// sure to escape any user input!
    ReplyMarkdown(String),

    /// Reply to the message with the given sticker and stop running handlers.
    ReplySticker(String),
}

#[async_trait]
pub trait BotHandlerFn<S: BotState>: Send + Sync {
    async fn run(&self, event: Event, state: State<S>) -> Result<Action, anyhow::Error>;
}

#[async_trait]
pub trait BotHandler<S: BotState>: Send + Sync + BotHandlerFn<S> {
    fn get_state(&self) -> &State<S>;
    fn set_state(&mut self, state: Arc<RwLock<S>>);
}

/// A handler for a specific chat ID. This is a wrapper around an async function
/// that takes a `ChatEvent` and returns a `ChatAction`.
pub struct Handler<S: BotState> {
    /// Wraps the async handler function.
    #[allow(clippy::type_complexity)]
    pub f: Box<dyn BotHandlerFn<S>>,

    // Box<dyn Fn(Event, State<S>) -> BoxFuture<'static, Result<Action, anyhow::Error>> + Send + Sync>,
    /// State related to this Chat ID
    pub state: State<S>,
}

impl<S: BotState> Handler<S> {
    pub fn new(func: Box<dyn BotHandlerFn<S>>) -> Self {
        Self {
            f: func,
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

#[async_trait]
impl<S: BotState> BotHandler<S> for Handler<S> {
    fn get_state(&self) -> &State<S> {
        &self.state
    }

    fn set_state(&mut self, state: Arc<RwLock<S>>) {
        self.state = State { state };
    }
}

#[async_trait]
impl<S: BotState> BotHandlerFn<S> for Handler<S> {
    async fn run(&self, event: Event, state: State<S>) -> Result<Action, anyhow::Error> {
        self.f.run(event, state).await
    }
}

pub struct HandlerFn<S: BotState> {
    #[allow(clippy::type_complexity)]
    pub f: Box<
        dyn Fn(Event, State<S>) -> BoxFuture<'static, Result<Action, anyhow::Error>> + Send + Sync,
    >,
}

impl<S: BotState> HandlerFn<S> {
    pub fn new<Func, Fut>(f: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(Event, State<S>) -> Fut,
        Fut: Send + 'static + Future<Output = Result<Action, anyhow::Error>>,
    {
        Self {
            f: Box::new(move |a, b| Box::pin(f(a, b))),
        }
    }
}

#[async_trait]
impl<S: BotState> BotHandlerFn<S> for HandlerFn<S> {
    async fn run(&self, event: Event, state: State<S>) -> Result<Action, anyhow::Error> {
        (self.f)(event, state).await
    }
}

impl<S> From<Box<dyn BotHandlerFn<S>>> for Box<dyn BotHandler<S>>
where
    S: BotState,
{
    fn from(func: Box<dyn BotHandlerFn<S>>) -> Box<dyn BotHandler<S>> {
        Box::new(Handler::new(func))
    }
}

impl<S, Func, Fut> From<Func> for Box<dyn BotHandlerFn<S>>
where
    S: BotState,
    Func: Send + Sync + 'static + Fn(Event, State<S>) -> Fut,
    Fut: Send + 'static + Future<Output = Result<Action, anyhow::Error>>,
{
    fn from(func: Func) -> Box<dyn BotHandlerFn<S>> {
        Box::new(HandlerFn::new(func))
    }
}
