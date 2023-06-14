use std::sync::Arc;

use async_trait::async_trait;
use futures::{future::BoxFuture, Future};
use tokio::sync::RwLock;

use crate::{Action, Event};

/// Bot states need to derive the `BotState` trait. You can use `#[derive(BotState)]` to do this.
pub trait BotState: Default + Clone + Send + Sync + 'static {}
impl BotState for () {}

/// Bot states are wrapped in an `Arc<RwLock<T>>` so that they can be shared between threads.
#[derive(Clone, Default)]
pub struct State<T: BotState> {
    state: Arc<RwLock<T>>,
}

impl<T: BotState> State<T> {
    pub fn new(state: T) -> Self {
        Self {
            state: Arc::new(RwLock::new(state)),
        }
    }

    /// Return a clone of the internal state wrapped in a new `Arc<RwLock<T>>`.
    pub async fn from(&self) -> Self {
        Self {
            state: Arc::new(RwLock::new((*self.state.read().await).clone())),
        }
    }

    /// Return a reference to the internal state.
    pub fn get(&self) -> &Arc<RwLock<T>> {
        &self.state
    }
}

/// BotHandlerFns are async functions that take an `Event` and a `State` and return an `Action`
#[async_trait]
pub trait BotHandlerFn<S: BotState>: Send + Sync {
    async fn run(&self, event: Event, state: State<S>) -> Result<Action, anyhow::Error>;
}

/// `BotHandler`s are handlers that can be registered with a `Router`. They must also implement the `BotHandlerFn` trait.
#[async_trait]
pub trait BotHandler<S: BotState>: Send + Sync + BotHandlerFn<S> {
    fn get_state(&self) -> &State<S>;
    fn set_state(&mut self, state: Arc<RwLock<S>>);
}

/// `Handler` is a concrete implementation of a `BotHandler`. It takes stores a `BotHandlerFn` and a `State`.
pub struct Handler<S: BotState> {
    /// Wraps the async handler function.
    #[allow(clippy::type_complexity)]
    pub f: Box<dyn BotHandlerFn<S>>,

    /// State related to the context in which it is called (e.g., a chat ID or a user ID)
    pub state: State<S>,
}

impl<S: BotState> Handler<S> {
    /// Create a new `Handler` from a `BotHandlerFn`.
    pub fn new(func: Box<dyn BotHandlerFn<S>>) -> Self {
        Self {
            f: func,
            state: State::default(),
        }
    }

    /// Attach a state to the handler.
    pub fn with_state(self, state: S) -> Self {
        Self {
            f: self.f,
            state: State::new(state),
        }
    }
}

/// Implement the `BotHandler` trait for `Handler`.
#[async_trait]
impl<S: BotState> BotHandler<S> for Handler<S> {
    fn get_state(&self) -> &State<S> {
        &self.state
    }

    fn set_state(&mut self, state: Arc<RwLock<S>>) {
        self.state = State { state };
    }
}

/// Implement the BotHandlerFn trait for Handler.
#[async_trait]
impl<S: BotState> BotHandlerFn<S> for Handler<S> {
    async fn run(&self, event: Event, state: State<S>) -> Result<Action, anyhow::Error> {
        self.f.run(event, state).await
    }
}

/// `HandlerFn` is a concrete implementation of a `BotHandlerFn`. It stores a boxed async function.
pub struct HandlerFn<S: BotState> {
    #[allow(clippy::type_complexity)]
    pub f: Box<
        dyn Fn(Event, State<S>) -> BoxFuture<'static, Result<Action, anyhow::Error>> + Send + Sync,
    >,
}

impl<S: BotState> HandlerFn<S> {
    /// Create a new `HandlerFn` from the given async function.
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

/// Implement the `BotHandlerFn` trait for `HandlerFn`.
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

/// Convert an async function into a `BotHandlerFn`.
impl<S, Func, Fut> From<Func> for Box<dyn BotHandler<S>>
where
    S: BotState,
    Func: Send + Sync + 'static + Fn(Event, State<S>) -> Fut,
    Fut: Send + 'static + Future<Output = Result<Action, anyhow::Error>>,
{
    fn from(func: Func) -> Box<dyn BotHandler<S>> {
        Box::new(Handler::new(Box::new(HandlerFn::new(func))))
    }
}
