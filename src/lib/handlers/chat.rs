use std::sync::Arc;

use anyhow::{anyhow, bail};
use futures::{future::BoxFuture, Future};
use tokio::sync::RwLock;

use crate::{
    api::{self, CallbackQuery, Message, Update},
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

/// `MessageEvent` represents a new or edited message.
#[derive(Debug, Clone)]
pub enum MessageEvent {
    New(Message),
    Edited(Message),
    Post(Message),
    EditedPost(Message),
    Callback(CallbackQuery),
    Unknown,
}

impl From<Update> for MessageEvent {
    fn from(update: Update) -> Self {
        if let Some(ref m) = update.message {
            Self::New(m.clone())
        } else if let Some(ref m) = update.edited_message {
            Self::Edited(m.clone())
        } else if let Some(ref m) = update.channel_post {
            Self::Post(m.clone())
        } else if let Some(ref m) = update.edited_channel_post {
            Self::EditedPost(m.clone())
        } else if let Some(ref c) = update.callback_query {
            Self::Callback(c.clone())
        } else {
            Self::Unknown
        }
    }
}

impl From<MessageEvent> for Message {
    fn from(event: MessageEvent) -> Self {
        match event {
            MessageEvent::New(msg) => msg,
            MessageEvent::Edited(msg) => msg,
            MessageEvent::Post(msg) => msg,
            MessageEvent::EditedPost(msg) => msg,
            MessageEvent::Callback(query) => query.message.unwrap(),
            MessageEvent::Unknown => {
                panic!("Bad MessageEvent::Unknown")
            }
        }
    }
}

impl From<MessageEvent> for CallbackQuery {
    fn from(event: MessageEvent) -> Self {
        match event {
            MessageEvent::Callback(query) => query,
            _ => {
                panic!("MessageEvent {:?} is not a CallbackQuery", event)
            }
        }
    }
}

impl ToString for MessageEvent {
    fn to_string(&self) -> String {
        match self {
            Self::New(msg) => msg.text.clone().unwrap(),
            Self::Edited(msg) => msg.text.clone().unwrap(),
            Self::Post(msg) => msg.text.clone().unwrap(),
            Self::EditedPost(msg) => msg.text.clone().unwrap(),
            Self::Callback(query) => query.data.clone().unwrap(),
            Self::Unknown => {
                panic!("Bad MessageEvent::Unknown")
            }
        }
    }
}

impl Event {
    /// Get a new or edited message from the event.
    pub fn message(&self) -> Result<&Message, anyhow::Error> {
        match &self.message {
            MessageEvent::New(msg) => Ok(msg),
            MessageEvent::Edited(msg) => Ok(msg),
            _ => Err(anyhow::anyhow!("MessageEvent is not a Message")),
        }
    }

    /// Get a new or edited post from the event.
    pub fn post(&self) -> Result<&Message, anyhow::Error> {
        match &self.message {
            MessageEvent::Post(msg) => Ok(msg),
            MessageEvent::EditedPost(msg) => Ok(msg),
            _ => Err(anyhow::anyhow!("MessageEvent is not a Post")),
        }
    }

    /// Get a new or edited message or post from the event.
    pub fn message_or_post(&self) -> Result<&Message, anyhow::Error> {
        match &self.message {
            MessageEvent::New(msg) => Ok(msg),
            MessageEvent::Edited(msg) => Ok(msg),
            MessageEvent::Post(msg) => Ok(msg),
            MessageEvent::EditedPost(msg) => Ok(msg),
            _ => Err(anyhow::anyhow!("MessageEvent is not a Message")),
        }
    }

    /// Get the user who sent the message.
    pub fn from_user(&self) -> Result<api::User, anyhow::Error> {
        match self.message.clone() {
            MessageEvent::New(msg)
            | MessageEvent::Edited(msg)
            | MessageEvent::Post(msg)
            | MessageEvent::EditedPost(msg) => {
                Ok(msg.from.ok_or(anyhow!("MessageEvent::New has no user"))?)
            }
            MessageEvent::Callback(query) => Ok(query.from),
            _ => Err(anyhow!("MessageEvent is not a Message")),
        }
    }

    /// Get a new message from the event.
    pub fn get_new_message(&self) -> Result<&Message, anyhow::Error> {
        match &self.message {
            MessageEvent::New(msg) => Ok(msg),
            _ => Err(anyhow!("MessageEvent is not a New Message")),
        }
    }

    /// Get an edited message from the event.
    pub fn get_edited_message(&self) -> Result<&Message, anyhow::Error> {
        match &self.message {
            MessageEvent::Edited(msg) => Ok(msg),
            _ => Err(anyhow!("MessageEvent is not an Edited Message")),
        }
    }

    /// Get a new post from the event.
    pub fn get_new_post(&self) -> Result<&Message, anyhow::Error> {
        match &self.message {
            MessageEvent::Post(msg) => Ok(msg),
            _ => Err(anyhow!("MessageEvent is not a New Post")),
        }
    }

    /// Get an edited post from the event.
    pub fn get_edited_post(&self) -> Result<&Message, anyhow::Error> {
        match &self.message {
            MessageEvent::EditedPost(msg) => Ok(msg),
            _ => Err(anyhow!("MessageEvent is not an Edited Post")),
        }
    }

    /// Get a callback query from the event.
    pub fn get_callback_query(&self) -> Result<&CallbackQuery, anyhow::Error> {
        match &self.message {
            MessageEvent::Callback(query) => Ok(query),
            _ => Err(anyhow!("MessageEvent is not a CallbackQuery")),
        }
    }

    pub fn chat_id(&self) -> i64 {
        match &self.message {
            MessageEvent::New(msg) => msg.chat.id,
            MessageEvent::Edited(msg) => msg.chat.id,
            MessageEvent::Post(msg) => msg.chat.id,
            MessageEvent::EditedPost(msg) => msg.chat.id,
            MessageEvent::Callback(query) => query.message.as_ref().unwrap().chat.id,
            MessageEvent::Unknown => {
                panic!("Bad MessageEvent::Unknown")
            }
        }
    }

    pub fn message_id(&self) -> i64 {
        match &self.message {
            MessageEvent::New(msg) => msg.message_id,
            MessageEvent::Edited(msg) => msg.message_id,
            MessageEvent::Post(msg) => msg.message_id,
            MessageEvent::EditedPost(msg) => msg.message_id,
            MessageEvent::Callback(query) => query.message.as_ref().unwrap().message_id,
            MessageEvent::Unknown => {
                panic!("Bad MessageEvent::Unknown")
            }
        }
    }

    /// Acknowledge a callback query.
    pub async fn acknowledge_callback(&self, text: Option<String>) -> anyhow::Result<bool> {
        let query_id = match &self.message {
            MessageEvent::Callback(query) => query.id.clone(),
            _ => {
                bail!("MessageEvent {:?} is not a CallbackQuery", self.message)
            }
        };

        let mut req = api::AnswerCallbackQueryRequest::new(query_id);
        if text.is_some() {
            req = req.with_text(text.unwrap());
        }

        self.api.answer_callback_query(&req).await
    }

    /// Remove the inline keyboard from a message.
    pub async fn remove_inline_keyboard(&self) -> anyhow::Result<Message> {
        let chat_id = self.chat_id();
        let message_id = self.message_id();

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
            .send_chat_action(&api::SendChatActionRequest::new(self.chat_id(), action))
            .await
    }

    /// Send a text message to the chat.
    pub async fn send_text(&self, text: impl Into<String>) -> anyhow::Result<Message> {
        self.api
            .send_message(&api::SendMessageRequest::new(self.chat_id(), text.into()))
            .await
    }

    /// Send a MarkdownV2 message to the chat.
    pub async fn send_markdown(&self, text: impl Into<String>) -> anyhow::Result<Message> {
        self.api
            .send_message(
                &api::SendMessageRequest::new(self.chat_id(), text.into())
                    .with_parse_mode(api::ParseMode::MarkdownV2),
            )
            .await
    }

    /// Edit the message with the given text (uses the parsemode of the message)
    pub async fn edit_message(&self, text: impl Into<String>) -> anyhow::Result<Message> {
        let chat_id = self.chat_id();
        let message_id = self.message_id();

        self.api
            .edit_message_text(&api::EditMessageTextRequest {
                base: api::EditMessageBase::new()
                    .with_chat_id(chat_id)
                    .with_message_id(message_id),
                text: text.into(),
            })
            .await
    }

    /// Send a sticker to the chat.
    pub async fn send_sticker(&self, sticker: impl Into<String>) -> anyhow::Result<Message> {
        self.api
            .send_sticker(&api::SendStickerRequest::new(
                self.chat_id(),
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
