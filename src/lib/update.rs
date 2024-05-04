use crate::api::{self, Document, PhotoSize};
use anyhow::anyhow;
use std::fmt;

/// `Update` represents a new update from Telegram
#[derive(Debug, Clone)]
pub enum Update {
    Message(api::Message),
    EditedMessage(api::Message),
    ChannelPost(api::Message),
    EditedChannelPost(api::Message),
    CallbackQuery(api::CallbackQuery),
    InlineQuery(api::InlineQuery),
    Unknown,
}

impl From<api::Update> for Update {
    fn from(update: api::Update) -> Self {
        if let Some(ref m) = update.message {
            Self::Message(m.clone())
        } else if let Some(ref m) = update.edited_message {
            Self::EditedMessage(m.clone())
        } else if let Some(ref m) = update.channel_post {
            Self::ChannelPost(m.clone())
        } else if let Some(ref m) = update.edited_channel_post {
            Self::EditedChannelPost(m.clone())
        } else if let Some(ref c) = update.callback_query {
            Self::CallbackQuery(c.clone())
        } else if let Some(ref c) = update.inline_query {
            Self::InlineQuery(c.clone())
        } else {
            Self::Unknown
        }
    }
}

impl From<Update> for api::Message {
    fn from(event: Update) -> Self {
        use Update::*;

        match event {
            Message(msg) => msg,
            EditedMessage(msg) => msg,
            ChannelPost(msg) => msg,
            EditedChannelPost(msg) => msg,
            CallbackQuery(query) => query.message.unwrap(),
            InlineQuery(_) | Unknown => {
                panic!("Bad Message::Unknown")
            }
        }
    }
}

impl From<Update> for api::CallbackQuery {
    fn from(event: Update) -> Self {
        match event {
            Update::CallbackQuery(query) => query,
            _ => {
                panic!("Message {:?} is not a CallbackQuery", event)
            }
        }
    }
}
impl fmt::Display for Update {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Update::*;
        match self {
            Message(msg) => write!(f, "{}", msg.text.clone().unwrap()),
            EditedMessage(msg) => write!(f, "{}", msg.text.clone().unwrap()),
            ChannelPost(msg) => write!(f, "{}", msg.text.clone().unwrap()),
            EditedChannelPost(msg) => write!(f, "{}", msg.text.clone().unwrap()),
            CallbackQuery(query) => write!(f, "{}", query.data.clone().unwrap()),
            InlineQuery(query) => write!(f, "{}", query.query.clone()),
            Unknown => {
                panic!("Bad Message::Unknown")
            }
        }
    }
}

impl Update {
    pub fn get_new(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::Message(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not a NewMessage"))
    }

    pub fn get_edited(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::EditedMessage(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not an EditedMessage"))
    }

    pub fn get_new_post(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::ChannelPost(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not a Post"))
    }

    pub fn get_edited_post(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::EditedChannelPost(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not an EditedPost"))
    }

    pub fn get_callback_query(&self) -> anyhow::Result<&api::CallbackQuery> {
        match self {
            Update::CallbackQuery(query) => Some(query),
            _ => None,
        }
        .ok_or(anyhow!("message is not a CallbackQuery"))
    }

    pub fn get_message_or_post(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::Message(msg) => Some(msg),
            Update::EditedMessage(msg) => Some(msg),
            Update::ChannelPost(msg) => Some(msg),
            Update::EditedChannelPost(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not a api::Message or Post"))
    }

    pub fn get_message(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::Message(msg) => Some(msg),
            Update::EditedMessage(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not a api::Message or Post"))
    }

    pub fn get_post(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::ChannelPost(msg) => Some(msg),
            Update::EditedChannelPost(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not a api::Message or Post"))
    }

    fn message(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::Message(msg) => Some(msg),
            Update::EditedMessage(msg) => Some(msg),
            Update::ChannelPost(msg) => Some(msg),
            Update::EditedChannelPost(msg) => Some(msg),
            Update::CallbackQuery(query) => Some(query.message.as_ref().unwrap()),
            Update::InlineQuery(_) | Update::Unknown => None,
        }
        .ok_or(anyhow!("message is not a api::Message"))
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

    pub fn photo(&self) -> anyhow::Result<&Vec<PhotoSize>> {
        self.message()
            .and_then(|msg| msg.photo.as_ref().ok_or(anyhow!("message has no photo")))
    }

    pub fn document(&self) -> anyhow::Result<&Document> {
        self.message().and_then(|msg| {
            msg.document
                .as_ref()
                .ok_or(anyhow!("message has no document"))
        })
    }

    pub fn data(&self) -> anyhow::Result<&str> {
        self.get_callback_query()
            .map(|query| query.data.as_ref().unwrap().as_str())
    }

    pub fn from_user(&self) -> anyhow::Result<&api::User> {
        use Update::*;
        match self {
            Message(msg) | EditedMessage(msg) | ChannelPost(msg) | EditedChannelPost(msg) => {
                msg.from.as_ref()
            }
            CallbackQuery(query) => Some(&query.from),
            _ => None,
        }
        .ok_or(anyhow!("message has no user"))
    }
}
