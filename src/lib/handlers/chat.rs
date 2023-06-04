use std::sync::Arc;

use futures::{future::BoxFuture, Future};
use tokio::sync::RwLock;

use crate::{
    api::{self, Message},
    messageevent::MessageEvent,
    API,
};

#[derive(Clone, Default)]
pub struct State<T: Clone> {
    state: Arc<RwLock<T>>,
}

impl<T: Clone> State<T> {
    pub async fn from(&self) -> Self {
        Self {
            state: Arc::new(RwLock::new((*self.state.read().await).clone())),
        }
    }

    pub fn get(&self) -> &Arc<RwLock<T>> {
        &self.state
    }
}

/// `Event` represents an event sent to a chat handler.
#[derive(Clone)]
pub struct Event {
    pub api: Arc<API>,
    pub message: MessageEvent,
}

impl Event {
    /// Acknowledge a callback query.
    pub async fn acknowledge_callback(&self, text: Option<String>) -> anyhow::Result<bool> {
        let query_id = self.message.query_id()?.to_string();

        let mut req = api::AnswerCallbackQueryRequest::new(query_id);
        if text.is_some() {
            req = req.with_text(text.unwrap());
        }

        self.api.answer_callback_query(&req).await
    }

    /// Remove the inline keyboard from a message.
    pub async fn remove_inline_keyboard(&self) -> anyhow::Result<Message> {
        let chat_id = self.message.chat_id()?;
        let message_id = self.message.message_id()?;

        // Remove the inline keyboard.
        self.api
            .edit_message_reply_markup(&api::EditMessageReplyMarkupRequest {
                base: api::EditMessageBase::new()
                    .with_chat_id(chat_id)
                    .with_message_id(message_id)
                    .with_reply_markup(api::ReplyMarkup::inline_keyboard_markup(vec![vec![]])),
            })
            .await
    }

    /// Send a chat action.
    pub async fn send_chat_action(&self, action: api::ChatAction) -> anyhow::Result<bool> {
        self.api
            .send_chat_action(&api::SendChatActionRequest::new(
                self.message.chat_id()?,
                action,
            ))
            .await
    }

    /// Send a text message to the chat.
    pub async fn send_text(&self, text: impl Into<String>) -> anyhow::Result<Message> {
        self.api
            .send_message(&api::SendMessageRequest::new(
                self.message.chat_id()?,
                text.into(),
            ))
            .await
    }

    /// Send a MarkdownV2 message to the chat.
    pub async fn send_markdown(&self, text: impl Into<String>) -> anyhow::Result<Message> {
        self.api
            .send_message(
                &api::SendMessageRequest::new(self.message.chat_id()?, text.into())
                    .with_parse_mode(api::ParseMode::MarkdownV2),
            )
            .await
    }

    /// Edit the message with the given text (uses the parsemode of the message)
    pub async fn edit_last_message(&self, text: impl Into<String>) -> anyhow::Result<Message> {
        self.edit_message(self.message.message_id()?, text).await
    }

    /// Edit the message with the given text (uses the parsemode of the message)
    pub async fn edit_message(
        &self,
        message_id: i64,
        text: impl Into<String>,
    ) -> anyhow::Result<Message> {
        let chat_id = self.message.chat_id()?;

        self.api
            .edit_message_text(&api::EditMessageTextRequest {
                base: api::EditMessageBase::new()
                    .with_chat_id(chat_id)
                    .with_message_id(message_id),
                text: text.into(),
            })
            .await
    }

    // Delete the last message
    pub async fn delete_last_message(&self) -> anyhow::Result<bool> {
        let chat_id = self.message.chat_id()?;
        let message_id = self.message.message_id()?;

        self.api
            .delete_message(&api::DeleteMessageRequest::new(chat_id, message_id))
            .await
    }

    // Delete a specific message
    pub async fn delete_message(&self, message_id: i64) -> anyhow::Result<bool> {
        let chat_id = self.message.chat_id()?;

        self.api
            .delete_message(&api::DeleteMessageRequest::new(chat_id, message_id))
            .await
    }

    /// Send a sticker to the chat.
    pub async fn send_sticker(&self, sticker: impl Into<String>) -> anyhow::Result<Message> {
        self.api
            .send_sticker(&api::SendStickerRequest::new(
                self.message.chat_id()?,
                sticker.into(),
            ))
            .await
    }
}

/// `Action` represents an action to take after handling a chat event.
#[derive(Debug, Clone)]
pub enum Action {
    /// Continue to the next handler.
    Next,

    /// Stop handling events.
    Done,

    /// Reply to the message with the given text and stop handling events. This
    /// is equivalent to `e.send_text(...)` followed by `Ok(Action::Done)`.
    ReplyText(String),

    /// Same as ReplyText, but with MarkdownV2 formatting. Make
    /// sure to escape any user input!
    ReplyMarkdown(String),

    /// Reply to the message with the given sticker and stop running handlers.
    ReplySticker(String),
}

/// A handler for a specific chat ID. This is a wrapper around an async function
/// that takes a `ChatEvent` and returns a `ChatAction`.
pub struct Handler<S: Clone> {
    /// Wraps the async handler function.
    #[allow(clippy::type_complexity)]
    pub f: Box<
        dyn Fn(Event, State<S>) -> BoxFuture<'static, Result<Action, anyhow::Error>> + Send + Sync,
    >,

    /// State related to this Chat ID
    pub state: State<S>,
}

impl<S: Clone> Handler<S>
where
    S: Default,
{
    pub fn new<Func, Fut>(func: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(Event, State<S>) -> Fut,
        Fut: Send + 'static + Future<Output = Result<Action, anyhow::Error>>,
    {
        Self {
            f: Box::new(move |a, b| Box::pin(func(a, b))),
            state: State {
                state: Arc::new(tokio::sync::RwLock::new(S::default())),
            },
        }
    }

    pub fn with_state(self, state: S) -> Self {
        Self {
            f: self.f,
            state: State {
                state: Arc::new(tokio::sync::RwLock::new(state)),
            },
        }
    }

    pub fn set_state(&mut self, state: Arc<RwLock<S>>) -> &mut Self {
        self.state = State { state };
        self
    }
}

impl<S, Func, Fut> From<Func> for Handler<S>
where
    S: Default + Clone,
    Func: Send + Sync + 'static + Fn(Event, State<S>) -> Fut,
    Fut: Send + 'static + Future<Output = Result<Action, anyhow::Error>>,
{
    fn from(func: Func) -> Self {
        Self::new(func)
    }
}

/// This handler logs every message received.
pub async fn log_handler<S>(e: Event, _: S) -> Result<Action, anyhow::Error> {
    match e.message {
        MessageEvent::New(message)
        | MessageEvent::Edited(message)
        | MessageEvent::Post(message)
        | MessageEvent::EditedPost(message) => {
            let chat_id = message.chat.id;
            let from = message.from.unwrap_or_default();
            let text = message.text.unwrap_or_default();

            info!("({}) Message from {}: {}", chat_id, from.first_name, text);

            Ok(Action::Next)
        }
        MessageEvent::Callback(query) => {
            let chat_id = query.message.unwrap_or_default().chat.id;
            let from = query.from;
            let data = query.data.unwrap_or_default();

            info!("({}) Callback from {}: {}", chat_id, from.first_name, data);

            Ok(Action::Next)
        }
        _ => Err(anyhow::anyhow!("Unknown message type")),
    }
}

pub async fn done_handler<S>(_: Event, _: S) -> Result<Action, anyhow::Error> {
    Ok(Action::Done)
}
