use std::sync::Arc;

use anyhow::Error;
use futures::{future::BoxFuture, Future};

use crate::{api::InlineQuery, API};

#[derive(Debug, Clone)]
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
pub struct Handler<S> {
    /// Wraps the async handler function.
    #[allow(clippy::type_complexity)]
    pub f: Box<dyn Fn(Event, S) -> BoxFuture<'static, Result<Action, Error>> + Send + Sync>,

    /// State related to this Chat ID
    pub state: S,
}

impl<S> Handler<S>
where
    S: Default,
{
    pub fn new<Func, Fut>(func: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(Event, S) -> Fut,
        Fut: Send + 'static + Future<Output = Result<Action, Error>>,
    {
        Self {
            f: Box::new(move |a, b| Box::pin(func(a, b))),
            state: S::default(),
        }
    }

    pub fn with_state(self, state: S) -> Self {
        Self { f: self.f, state }
    }
}

impl<S, Func, Fut> From<Func> for Handler<S>
where
    S: Default,
    Func: Send + Sync + 'static + Fn(Event, S) -> Fut,
    Fut: Send + 'static + Future<Output = Result<Action, Error>>,
{
    fn from(func: Func) -> Self {
        Self::new(func)
    }
}
