use std::sync::Arc;

use anyhow::Error;
use futures::{future::BoxFuture, Future};
use tokio::sync::RwLock;

use crate::{api::InlineQuery, API};

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

#[derive(Clone)]
pub struct Event {
    pub api: Arc<API>,
    pub query: InlineQuery,
}

#[derive(Debug, Clone)]
pub enum Action {
    Next,
    Done,
    ReplyText(String, String),
}

/// A handler for a specific chat ID. This is a wrapper around an async function
/// that takes a `ChatEvent` and returns a `ChatAction`.
pub struct Handler<S: Clone> {
    /// Wraps the async handler function.
    #[allow(clippy::type_complexity)]
    pub f: Box<dyn Fn(Event, State<S>) -> BoxFuture<'static, Result<Action, Error>> + Send + Sync>,

    /// State related to this Chat ID
    pub state: State<S>,
}

impl<S> Handler<S>
where
    S: Clone + Default,
{
    pub fn new<Func, Fut>(func: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(Event, State<S>) -> Fut,
        Fut: Send + 'static + Future<Output = Result<Action, Error>>,
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
    Fut: Send + 'static + Future<Output = Result<Action, Error>>,
{
    fn from(func: Func) -> Self {
        Self::new(func)
    }
}
