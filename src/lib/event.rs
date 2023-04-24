use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{Message, Update};

#[derive(Debug, Clone)]
pub enum Event {
    NewMessage(i64, Message),
    EditedMessage(i64, Message),
    ChannelPost(i64, Message),
    EditedChannelPost(i64, Message),
    BadUpdate(i64, String),
}

impl From<Update> for Event {
    fn from(update: Update) -> Self {
        if let Some(message) = update.message {
            Self::NewMessage(update.update_id, message)
        } else if let Some(message) = update.edited_message {
            Self::EditedMessage(update.update_id, message)
        } else if let Some(message) = update.channel_post {
            Self::ChannelPost(update.update_id, message)
        } else if let Some(message) = update.edited_channel_post {
            Self::EditedChannelPost(update.update_id, message)
        } else {
            Self::BadUpdate(update.update_id, format!("Unknown update: {update:?}"))
        }
    }
}

pub trait Request: Serialize + Clone + Send + Sync {}
pub trait Response: DeserializeOwned + Clone + Send + Sync {}
