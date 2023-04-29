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
pub struct Event<T>
where
    T: TelegramClient,
{
    pub api: Arc<API<T>>,
    pub message: MessageEvent,
}

#[derive(Debug, Clone)]
pub enum Op {
    ReplyText(String),
    ReplySticker(String),
    None,
}

impl<T: Into<String>> From<T> for Op {
    fn from(s: T) -> Self {
        Op::ReplyText(s.into())
    }
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
pub enum Action<T> {
    Next(T),
    Done(T),
}

/// A handler for a specific chat ID. This is a wrapper around an async function
/// that takes a `ChatEvent` and returns a `ChatAction`.
pub struct Handler<R, S, T>
where
    R: Into<Action<Op>>,
    T: TelegramClient,
{
    /// Wraps the async handler function.
    #[allow(clippy::type_complexity)]
    pub f: Box<dyn Fn(Event<T>, S) -> BoxFuture<'static, Result<R, Error>> + Send + Sync>,

    /// State related to this Chat ID
    pub state: S,
}

impl<R, S, T> Handler<R, S, T>
where
    R: Into<Action<Op>>,
    S: Default,
    T: TelegramClient,
{
    pub fn new<Func, Fut>(func: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(Event<T>, S) -> Fut,
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

impl<R, S, T, Func, Fut> From<Func> for Handler<R, S, T>
where
    R: Into<Action<Op>>,
    S: Default,
    T: TelegramClient,
    Func: Send + Sync + 'static + Fn(Event<T>, S) -> Fut,
    Fut: Send + 'static + Future<Output = Result<R, Error>>,
{
    fn from(func: Func) -> Self {
        Self::new(func)
    }
}

/// This handler logs every message received.
pub async fn log_handler<T, S>(e: Event<T>, _: S) -> Result<Action<Op>, Error>
where
    T: TelegramClient,
{
    match e.message {
        MessageEvent::New(message) | MessageEvent::Edited(message) => {
            let chat_id = message.chat.id;
            let from = message.from.unwrap();
            let text = message.text.unwrap_or_default();

            info!("({}) Message from {}: {}", chat_id, from.first_name, text);

            Ok(Action::Next(Op::None))
        }
    }
}
