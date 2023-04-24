use crate::{Client, Event};
use anyhow::Result;
use futures::future::BoxFuture;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Handler error: {0}")]
    Failed(String),
}

pub enum Action {
    Next,
    Done,
}

pub struct EventHandler<R, S = ()>
where
    R: Into<Action>,
{
    /// Wraps the async handler function.
    #[allow(clippy::type_complexity)]
    f: Box<dyn Fn(Event, S) -> BoxFuture<'static, Result<R, EventError>> + Send + Sync>,

    /// Any arbitrary state
    state: S,
}

impl<R, S: Clone> EventHandler<R, S>
where
    R: Into<Action>,
{
    pub fn with_state(self, state: &S) -> Self {
        Self {
            f: self.f,
            state: state.clone(),
        }
    }
}

pub struct Router {
    client: Client,
}

impl Router {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}
