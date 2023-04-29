use std::sync::Arc;

use futures::{future::BoxFuture, Future};
use thiserror::Error;

use crate::{InlineQuery, TelegramClient, API};

#[derive(Debug, Clone)]
pub struct Event<T>
where
    T: TelegramClient,
{
    pub api: Arc<API<T>>,
    pub query: InlineQuery,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Handler error: {0}")]
    Failed(String),
}

impl<T: Into<String>> From<T> for Error {
    fn from(s: T) -> Self {
        Error::Failed(s.into())
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    Next,
    Done,
    ReplyText(String),
}

/// A handler for a specific chat ID. This is a wrapper around an async function
/// that takes a `ChatEvent` and returns a `ChatAction`.
pub struct Handler<S, T>
where
    T: TelegramClient,
{
    /// Wraps the async handler function.
    #[allow(clippy::type_complexity)]
    pub f: Box<dyn Fn(Event<T>, S) -> BoxFuture<'static, Result<Action, Error>> + Send + Sync>,

    /// State related to this Chat ID
    pub state: S,
}

impl<S, T> Handler<S, T>
where
    S: Default,
    T: TelegramClient,
{
    pub fn new<Func, Fut>(func: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(Event<T>, S) -> Fut,
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

impl<S, T, Func, Fut> From<Func> for Handler<S, T>
where
    S: Default,
    T: TelegramClient,
    Func: Send + Sync + 'static + Fn(Event<T>, S) -> Fut,
    Fut: Send + 'static + Future<Output = Result<Action, Error>>,
{
    fn from(func: Func) -> Self {
        Self::new(func)
    }
}
