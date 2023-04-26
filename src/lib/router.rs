use std::{cmp::max, collections::HashMap, sync::Arc};

use crate::{Client, GetUpdatesRequest, Message, UpdateEvent, API};
use anyhow::Result;
use futures::{future::BoxFuture, Future};
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub enum MessageEvent {
    New(Message),
    Edited(Message),
}

#[derive(Debug, Clone)]
pub struct ChatEvent {
    pub api: Arc<API>,
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
pub struct ChatHandler<R, S = ()>
where
    R: Into<Action<ChatAction>>,
{
    /// Wraps the async handler function.
    #[allow(clippy::type_complexity)]
    f: Box<dyn Fn(ChatEvent, S) -> BoxFuture<'static, Result<R, Error>> + Send + Sync>,

    /// State related to this Chat ID
    state: S,
}

impl<R, S: Default> ChatHandler<R, S>
where
    R: Into<Action<ChatAction>>,
{
    pub fn new<Func: Send + Sync, Fut>(func: Func) -> Self
    where
        Func: Send + 'static + Fn(ChatEvent, S) -> Fut,
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

// Handler routing:
//  - by chat ID
//  - by user
//  - by message type
//  - by message text regex
//
// create filtering functions for each of these, and then compose them together

pub struct Router<R, S>
where
    R: Into<Action<ChatAction>>,
{
    api: Arc<API>,
    chat_handler: Option<ChatHandler<R, S>>,
    chat_state: HashMap<i64, S>,
}

impl<R: Into<Action<ChatAction>>, S: Clone> Router<R, S> {
    pub fn new(client: Client) -> Self {
        Self {
            api: Arc::new(API::new(client)),
            chat_handler: None,
            chat_state: HashMap::new(),
        }
    }

    pub fn add_chat_handler(&mut self, h: ChatHandler<R, S>) {
        self.chat_handler = Some(h);
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

                        let state = self
                            .chat_state
                            .entry(chat_id)
                            .or_insert(self.chat_handler.as_ref().unwrap().state.clone());

                        let reply = (self.chat_handler.as_ref().unwrap().f)(
                            ChatEvent {
                                api: Arc::clone(&self.api),
                                message: MessageEvent::New(message),
                            },
                            state.clone(),
                        )
                        .await
                        .unwrap();

                        match reply.into() {
                            Action::Next(ChatAction::ReplyText(text)) => {
                                self.api
                                    .send_message(&crate::SendMessageRequest {
                                        chat_id,
                                        text,
                                        reply_to_message_id: None,
                                    })
                                    .await
                                    .expect("Failed to send message");
                            }
                            Action::Next(ChatAction::ReplySticker(sticker)) => {
                                self.api
                                    .send_sticker(&crate::SendStickerRequest::new(chat_id, sticker))
                                    .await
                                    .expect("Failed to send message");
                            }
                            Action::Next(ChatAction::None) => {}
                            Action::Done(_) => {}
                        }
                    }
                    _ => {
                        warn!("Unhandled update: {update:?}");
                    }
                }
            }
        }
    }
}
