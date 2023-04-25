use std::{cmp::max, collections::HashMap};

use crate::{Client, GetUpdatesRequest, Message, UpdateEvent, API};
use anyhow::Result;
use futures::future::BoxFuture;
use thiserror::Error;

pub enum ChatEvent {
    NewMessage(Message),
    EditedMessage(Message),
}

#[derive(Error, Debug)]
pub enum ChatError {
    #[error("Handler error: {0}")]
    Failed(String),
}

pub enum ChatAction {
    ReplyText(String),
    ReplySticker(String),
    Next,
    Done,
}

pub struct ChatHandler<R, S = ()>
where
    R: Into<ChatAction>,
{
    /// Wraps the async handler function.
    #[allow(clippy::type_complexity)]
    f: Box<dyn Fn(ChatEvent, S) -> BoxFuture<'static, Result<R, ChatError>> + Send + Sync>,

    /// Any arbitrary state
    state: Option<S>,
}

impl<R, S: Clone> ChatHandler<R, S>
where
    R: Into<ChatAction>,
{
    pub fn with_state(self, state: &S) -> Self {
        Self {
            f: self.f,
            state: Some(state.clone()),
        }
    }
}

// Handler routing:
//  - by chat ID
//  - by user
//  - by message type
//  - by message text regex
//
// create filtering functions for each of these, and then compose them together

pub struct Router<R, S>
where
    R: Into<ChatAction>,
{
    api: API,
    chat_handler: Option<ChatHandler<R, S>>,
    chat_state: HashMap<i64, S>,
}

impl<R: Into<ChatAction>, S> Router<R, S> {
    pub fn set_chat_handler<F>(&mut self, h: ChatHandler<R, S>) {
        self.chat_handler = Some(h);
    }
}

impl<R: Into<ChatAction>, S: Default + Clone> Router<R, S> {
    pub fn new(client: Client) -> Self {
        Self {
            api: API::new(client),
            chat_handler: None,
            chat_state: HashMap::new(),
        }
    }

    pub async fn start(&mut self) {
        let mut last_update_id = 0;

        loop {
            debug!("last_update_id = {}", last_update_id);
            let updates = self
                .api
                .get_update_events(
                    &GetUpdatesRequest::new()
                        .with_timeout(60)
                        .with_offset(last_update_id + 1),
                )
                .await
                .unwrap();

            for update in updates {
                match update {
                    UpdateEvent::NewMessage(id, message) => {
                        last_update_id = max(last_update_id, id);
                        let from = message.from.clone().unwrap();
                        let text = message.text.clone().unwrap();
                        let chat_id = message.chat.id;

                        info!("({}) Message from {}: {}", chat_id, from.first_name, text);

                        let state = self.chat_state.entry(chat_id).or_insert(S::default());
                        (self.chat_handler.as_ref().unwrap().f)(
                            ChatEvent::NewMessage(message),
                            state.clone(),
                        )
                        .await
                        .unwrap();
                    }
                    _ => {
                        warn!("Unhandled update: {update:?}");
                    }
                }
            }
        }
    }
}
