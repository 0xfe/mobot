use std::sync::Arc;

use futures::{future::BoxFuture, Future};
use thiserror::Error;

use crate::{api::Message, API};

#[derive(Debug, Clone)]
pub enum MessageEvent {
    New(Message),
    Edited(Message),
}

#[derive(Debug, Clone)]
pub struct Event {
    pub api: Arc<API>,
    pub message: MessageEvent,
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

pub enum Action {
    Next,
    Done,
    ReplyText(String),
    ReplySticker(String),
}

/// A handler for a specific chat ID. This is a wrapper around an async function
/// that takes a `ChatEvent` and returns a `ChatAction`.
pub struct Handler<S> {
    /// Wraps the async handler function.
    #[allow(clippy::type_complexity)]
    pub f: Box<dyn Fn(Event, S) -> BoxFuture<'static, Result<Action, anyhow::Error>> + Send + Sync>,

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
        Fut: Send + 'static + Future<Output = Result<Action, anyhow::Error>>,
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
