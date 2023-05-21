use derive_more::{From, FromStr, Into};
use serde::{Deserialize, Serialize};

use crate::{Request, API};

use super::{chat::Chat, sticker::Sticker, user::User};

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    /// Unique message identifier inside this chat
    pub message_id: i64,

    /// Sender, empty for messages sent to channels
    pub from: Option<User>,

    /// Date the message was sent in Unix time
    pub date: i64,

    /// Message text
    pub text: Option<String>,

    /// Conversation the message belongs to
    /// - For sent messages, the first available identifier of the chat
    /// - For messages forwarded to the chat, the identifier of the original chat
    /// - For messages in channels, the identifier of the channel is contained in the `chat_id` field
    pub chat: Chat,

    /// For forwarded messages, sender of the original message
    pub forward_from: Option<User>,

    /// For messages forwarded from channels, information about the original channel
    pub forward_from_chat: Option<Chat>,

    /// For messages forwarded from channels, identifier of the original message in the channel
    pub forward_from_message_id: Option<i64>,

    /// For messages forwarded from channels, signature of the post author if present
    pub forward_signature: Option<String>,

    /// Sender's name for messages forwarded from users who disallow adding a link to their account in forwarded messages
    pub forward_sender_name: Option<String>,

    /// For forwarded messages, date the original message was sent in Unix time
    pub forward_date: Option<i64>,

    /// For replies, the original message. Note that the Message object in this field will not contain further `reply_to_message` fields even if it itself is a reply.
    pub reply_to_message: Option<i64>,

    /// Sticker for messages with a sticker
    pub sticker: Option<Sticker>,
}

impl Message {
    pub fn new(from: impl Into<String>, text: impl Into<String>) -> Self {
        let from = from.into();

        Self {
            from: Some(User {
                username: Some(from.clone()),
                first_name: from.clone(),
                ..Default::default()
            }),
            text: Some(text.into()),
            chat: Chat {
                chat_type: String::from("private"),
                username: Some(from.clone()),
                first_name: Some(from),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Clone, Into, FromStr, From)]
pub struct ParseMode(String);

impl Default for ParseMode {
    fn default() -> Self {
        Self(String::from("MarkdownV2"))
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct SendMessageRequest {
    /// Unique identifier for the target chat or username of the target
    pub chat_id: i64,

    /// Text of the message to be sent
    pub text: String,

    /// If the message is a reply, ID of the original message
    pub reply_to_message_id: Option<i64>,

    /// Parse mode for the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<String>,
}

impl Request for SendMessageRequest {}

impl API {
    pub async fn send_message(&self, req: &SendMessageRequest) -> anyhow::Result<Message> {
        self.client.post("sendMessage", req).await
    }
}
