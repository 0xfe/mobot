use anyhow::bail;
use async_trait::async_trait;

use crate::{
    handler::{BotHandlerFn, BotState},
    Action, Event, State,
};

pub struct AuthHandler {
    pub authorized_users: Vec<String>,
}

impl AuthHandler {
    pub fn new(authorized_users: Vec<String>) -> Self {
        Self { authorized_users }
    }
}

#[async_trait]
impl<S: BotState> BotHandlerFn<S> for AuthHandler {
    async fn run(&self, event: Event, _: State<S>) -> Result<Action, anyhow::Error> {
        if !self.authorized_users.contains(
            event
                .update
                .from_user()?
                .username
                .as_ref()
                .ok_or(anyhow::anyhow!("No username"))?,
        ) {
            bail!("Unauthorized user")
        }

        Ok(Action::Next)
    }
}
