use mobot_derive::BotRequest;
use serde::{Deserialize, Serialize};

use super::{message::Message, ReplyParameters, API};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Sticker {
    /// Unique identifier for this file
    pub file_id: String,

    /// Sticker width
    pub width: i64,

    /// Sticker height
    pub height: i64,

    /// True, if the sticker is animated
    pub is_animated: bool,

    /// Emoji associated with the sticker
    pub emoji: Option<String>,

    /// Name of the sticker set to which the sticker belongs
    pub set_name: Option<String>,

    /// File size
    pub file_size: Option<i64>,
}

#[derive(Debug, Serialize, Clone, BotRequest)]
pub struct SendStickerRequest {
    /// Unique identifier for the target chat or username of the target
    pub chat_id: i64,

    /// Sticker to send. Pass a file_id as String to send a file that
    pub sticker: String,

    /// Sends the message silently. Users will receive a notification with
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,

    /// If the message is a reply, ID of the original message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
}

impl SendStickerRequest {
    pub fn new(chat_id: i64, sticker: String) -> Self {
        Self {
            chat_id,
            sticker,
            disable_notification: None,
            reply_parameters: None,
        }
    }

    pub fn with_reply_parameters(mut self, reply_parameters: ReplyParameters) -> Self {
        self.reply_parameters = Some(reply_parameters);
        self
    }
}

impl API {
    pub async fn send_sticker(&self, req: &SendStickerRequest) -> anyhow::Result<Message> {
        self.client.post("sendSticker", req).await
    }
}
