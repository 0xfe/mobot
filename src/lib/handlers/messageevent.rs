use crate::api::{self, CallbackQuery, Message, Update};
use anyhow::anyhow;

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
        use MessageEvent::*;

        match event {
            New(msg) => msg,
            Edited(msg) => msg,
            Post(msg) => msg,
            EditedPost(msg) => msg,
            Callback(query) => query.message.unwrap(),
            Unknown => {
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
        use MessageEvent::*;
        match self {
            New(msg) => msg.text.clone().unwrap(),
            Edited(msg) => msg.text.clone().unwrap(),
            Post(msg) => msg.text.clone().unwrap(),
            EditedPost(msg) => msg.text.clone().unwrap(),
            Callback(query) => query.data.clone().unwrap(),
            Unknown => {
                panic!("Bad MessageEvent::Unknown")
            }
        }
    }
}

impl MessageEvent {
    pub fn get_new(&self) -> anyhow::Result<&Message> {
        match self {
            MessageEvent::New(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not a NewMessage"))
    }

    pub fn get_edited(&self) -> anyhow::Result<&Message> {
        match self {
            MessageEvent::Edited(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not an EditedMessage"))
    }

    pub fn get_new_post(&self) -> anyhow::Result<&Message> {
        match self {
            MessageEvent::Post(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not a Post"))
    }

    pub fn get_edited_post(&self) -> anyhow::Result<&Message> {
        match self {
            MessageEvent::EditedPost(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not an EditedPost"))
    }

    pub fn get_callback_query(&self) -> anyhow::Result<&CallbackQuery> {
        match self {
            MessageEvent::Callback(query) => Some(query),
            _ => None,
        }
        .ok_or(anyhow!("message is not a CallbackQuery"))
    }

    pub fn get_message_or_post(&self) -> anyhow::Result<&Message> {
        match self {
            MessageEvent::New(msg) => Some(msg),
            MessageEvent::Edited(msg) => Some(msg),
            MessageEvent::Post(msg) => Some(msg),
            MessageEvent::EditedPost(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not a Message or Post"))
    }

    pub fn get_message(&self) -> anyhow::Result<&Message> {
        match self {
            MessageEvent::New(msg) => Some(msg),
            MessageEvent::Edited(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not a Message or Post"))
    }

    pub fn get_post(&self) -> anyhow::Result<&Message> {
        match self {
            MessageEvent::Post(msg) => Some(msg),
            MessageEvent::EditedPost(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not a Message or Post"))
    }

    fn message(&self) -> anyhow::Result<&Message> {
        match self {
            MessageEvent::New(msg) => Some(msg),
            MessageEvent::Edited(msg) => Some(msg),
            MessageEvent::Post(msg) => Some(msg),
            MessageEvent::EditedPost(msg) => Some(msg),
            MessageEvent::Callback(query) => Some(query.message.as_ref().unwrap()),
            MessageEvent::Unknown => None,
        }
        .ok_or(anyhow!("message is not a Message"))
    }

    pub fn chat_id(&self) -> anyhow::Result<i64> {
        self.message().map(|msg| msg.chat.id)
    }

    pub fn message_id(&self) -> anyhow::Result<i64> {
        self.message().map(|msg| msg.message_id)
    }

    pub fn query_id(&self) -> anyhow::Result<&str> {
        self.get_callback_query().map(|query| query.id.as_str())
    }

    pub fn text(&self) -> anyhow::Result<&str> {
        self.message().and_then(|msg| {
            msg.text
                .as_ref()
                .ok_or(anyhow!("message has no text"))
                .map(|s| s.as_str())
        })
    }

    pub fn data(&self) -> anyhow::Result<&str> {
        self.get_callback_query()
            .map(|query| query.data.as_ref().unwrap().as_str())
    }

    pub fn from_user(&self) -> anyhow::Result<&api::User> {
        use MessageEvent::*;
        match self {
            New(msg) | Edited(msg) | Post(msg) | EditedPost(msg) => msg.from.as_ref(),
            Callback(query) => Some(&query.from),
            _ => None,
        }
        .ok_or(anyhow!("message has no user"))
    }
}
