use std::sync::Arc;

use futures::{future::BoxFuture, Future};
use thiserror::Error;

use crate::{Message, TelegramClient, API};

#[derive(Debug, Clone)]
pub enum MessageEvent {
    New(Message),
    Edited(Message),
}

#[derive(Debug, Clone)]
pub struct ChatEvent<T>
where
    T: TelegramClient,
{
    pub api: Arc<API<T>>,
    pub message: MessageEvent,
}

#[derive(Debug, Clone)]
pub enum ChatAction {
    ReplyText(String),
    ReplySticker(String),
    None,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Handler error: {0}")]
    Failed(String),
}

#[derive(Debug, Clone)]
pub enum Action<T> {
    Next(T),
    Done(T),
}

/// A handler for a specific chat ID. This is a wrapper around an async function
/// that takes a `ChatEvent` and returns a `ChatAction`.
pub struct ChatHandler<R, S, T>
where
    R: Into<Action<ChatAction>>,
    T: TelegramClient,
{
    /// Wraps the async handler function.
    #[allow(clippy::type_complexity)]
    pub f: Box<dyn Fn(ChatEvent<T>, S) -> BoxFuture<'static, Result<R, Error>> + Send + Sync>,

    /// State related to this Chat ID
    pub state: S,
}

impl<R, S, T> ChatHandler<R, S, T>
where
    R: Into<Action<ChatAction>>,
    S: Default,
    T: TelegramClient,
{
    pub fn new<Func, Fut>(func: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(ChatEvent<T>, S) -> Fut,
        Fut: Send + 'static + Future<Output = Result<R, Error>>,
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
