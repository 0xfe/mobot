use std::marker::PhantomData;

use anyhow::bail;
use async_trait::async_trait;

use crate::{
    handler::{BotHandlerFn, BotState},
    Action, Event, State,
};

/// This is a basic implementation of a handler that checks if the user is authorized.
pub struct AuthHandler<S> {
    pub authorized_users: Vec<String>,
    pub phanthom_data: PhantomData<S>,
}

impl<S> AuthHandler<S> {
    pub fn new(authorized_users: Vec<String>) -> Self {
        Self {
            authorized_users,
            phanthom_data: PhantomData,
        }
    }
}

#[async_trait]
impl<S: BotState> BotHandlerFn<S> for AuthHandler<S> {
    async fn run(&self, event: Event, _: State<S>) -> Result<Action, anyhow::Error> {
        if !self.authorized_users.contains(
            event
                .update
                .from_user()?
                .username
                .as_ref()
                .ok_or(anyhow::anyhow!("No username"))?,
        ) {
            bail!(
                "Unauthorized user: {}",
                event
                    .update
                    .from_user()?
                    .username
                    .as_ref()
                    .unwrap_or(&"__unknown__".to_string())
            );
        }

        Ok(Action::Next)
    }
}

pub fn auth_handler<S: BotState>(authorized_users: Vec<String>) -> Box<dyn BotHandlerFn<S>> {
    Box::new(AuthHandler::new(authorized_users))
}
