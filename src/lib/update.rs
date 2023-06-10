use crate::api;
use anyhow::anyhow;

/// `Message` represents a new or edited message.
#[derive(Debug, Clone)]
pub enum Update {
    New(api::Message),
    Edited(api::Message),
    Post(api::Message),
    EditedPost(api::Message),
    Callback(api::CallbackQuery),
    InlineQuery(api::InlineQuery),
    Unknown,
}

impl From<api::Update> for Update {
    fn from(update: api::Update) -> Self {
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
            New(msg) => msg,
            Edited(msg) => msg,
            Post(msg) => msg,
            EditedPost(msg) => msg,
            Callback(query) => query.message.unwrap(),
            InlineQuery(_) | Unknown => {
                panic!("Bad Message::Unknown")
            }
        }
    }
}

impl From<Update> for api::CallbackQuery {
    fn from(event: Update) -> Self {
        match event {
            Update::Callback(query) => query,
            _ => {
                panic!("Message {:?} is not a CallbackQuery", event)
            }
        }
    }
}

impl ToString for Update {
    fn to_string(&self) -> String {
        use Update::*;
        match self {
            New(msg) => msg.text.clone().unwrap(),
            Edited(msg) => msg.text.clone().unwrap(),
            Post(msg) => msg.text.clone().unwrap(),
            EditedPost(msg) => msg.text.clone().unwrap(),
            Callback(query) => query.data.clone().unwrap(),
            InlineQuery(query) => query.query.clone(),
            Unknown => {
                panic!("Bad Message::Unknown")
            }
        }
    }
}

impl Update {
    pub fn get_new(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::New(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not a NewMessage"))
    }

    pub fn get_edited(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::Edited(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not an EditedMessage"))
    }

    pub fn get_new_post(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::Post(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not a Post"))
    }

    pub fn get_edited_post(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::EditedPost(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not an EditedPost"))
    }

    pub fn get_callback_query(&self) -> anyhow::Result<&api::CallbackQuery> {
        match self {
            Update::Callback(query) => Some(query),
            _ => None,
        }
        .ok_or(anyhow!("message is not a CallbackQuery"))
    }

    pub fn get_message_or_post(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::New(msg) => Some(msg),
            Update::Edited(msg) => Some(msg),
            Update::Post(msg) => Some(msg),
            Update::EditedPost(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not a api::Message or Post"))
    }

    pub fn get_message(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::New(msg) => Some(msg),
            Update::Edited(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not a api::Message or Post"))
    }

    pub fn get_post(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::Post(msg) => Some(msg),
            Update::EditedPost(msg) => Some(msg),
            _ => None,
        }
        .ok_or(anyhow!("message is not a api::Message or Post"))
    }

    fn message(&self) -> anyhow::Result<&api::Message> {
        match self {
            Update::New(msg) => Some(msg),
            Update::Edited(msg) => Some(msg),
            Update::Post(msg) => Some(msg),
            Update::EditedPost(msg) => Some(msg),
            Update::Callback(query) => Some(query.message.as_ref().unwrap()),
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

    pub fn data(&self) -> anyhow::Result<&str> {
        self.get_callback_query()
            .map(|query| query.data.as_ref().unwrap().as_str())
    }

    pub fn from_user(&self) -> anyhow::Result<&api::User> {
        use Update::*;
        match self {
            New(msg) | Edited(msg) | Post(msg) | EditedPost(msg) => msg.from.as_ref(),
            Callback(query) => Some(&query.from),
            _ => None,
        }
        .ok_or(anyhow!("message has no user"))
    }
}
