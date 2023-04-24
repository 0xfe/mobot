use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SendStickerRequest {
    /// Unique identifier for the target chat or username of the target
    pub chat_id: i64,

    /// Sticker to send. Pass a file_id as String to send a file that
    pub sticker: String,

    /// Sends the message silently. Users will receive a notification with
    pub disable_notification: Option<bool>,

    /// If the message is a reply, ID of the original message
    pub reply_to_message_id: Option<i64>,
}

impl SendStickerRequest {
    pub fn new(chat_id: i64, sticker: String) -> Self {
        Self {
            chat_id,
            sticker,
            disable_notification: None,
            reply_to_message_id: None,
        }
    }

    pub fn with_disable_notification(mut self, disable_notification: bool) -> Self {
        self.disable_notification = Some(disable_notification);
        self
    }

    pub fn with_reply_to_message_id(mut self, reply_to_message_id: i64) -> Self {
        self.reply_to_message_id = Some(reply_to_message_id);
        self
    }
}
