use serde::{Deserialize, Serialize};

use crate::{Request, API};

use super::{chat::Chat, sticker::Sticker, user::User};

/// `Message` represents a message sent in a chat. It can be a text message, a sticker, a photo, etc.
/// <https://core.telegram.org/bots/api#message>
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
    /// Creates a new `Message` with the given `text` and `from` fields.
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ParseMode {
    #[serde(rename = "MarkdownV2")]
    MarkdownV2,
    #[serde(rename = "Markdown")]
    Markdown,
    #[serde(rename = "HTML")]
    HTML,
    #[serde(rename = "Text")]
    Text,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct KeyboardButton {
    /// Text of the button. If none of the optional fields are used, it will be sent as a message when the button is pressed
    pub text: String,
    // Optional fields omitted
}

impl<T: Into<String>> From<T> for KeyboardButton {
    fn from(text: T) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct InlineKeyboardButton {
    /// Label text on the button
    pub text: String,

    /// HTTP or tg:// url to be opened when button is pressed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Callback data to be sent in a callback query to the bot when button is pressed, 1-64 bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_data: Option<String>,
}

impl<T: Into<String>> From<T> for InlineKeyboardButton {
    fn from(text: T) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }
}

impl InlineKeyboardButton {
    pub fn with_callback_data<T: Into<String>>(mut self, callback_data: T) -> Self {
        self.callback_data = Some(callback_data.into());
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ReplyMarkup {
    InlineKeyboardMarkup {
        /// Array of button rows, each represented by an Array of KeyboardButton objects
        inline_keyboard: Vec<Vec<InlineKeyboardButton>>,

        /// Requests clients to resize the keyboard vertically for optimal fit
        resize_keyboard: bool,

        /// Requests clients to hide the keyboard as soon as it's been used
        one_time_keyboard: bool,

        /// Use this parameter if you want to show the keyboard to specific users only
        selective: bool,

        /// The placeholder to be shown in the input field when the keyboard is active; 1-64 characters
        #[serde(skip_serializing_if = "Option::is_none")]
        input_field_placeholder: Option<String>,

        /// Requests clients to always show the keyboard in the chat (users may not otherwise see the keyboard)
        is_persistent: bool,
    },
    ReplyKeyboardMarkup {
        /// Array of button rows, each represented by an Array of KeyboardButton objects
        keyboard: Vec<Vec<KeyboardButton>>,

        /// Requests clients to resize the keyboard vertically for optimal fit
        resize_keyboard: bool,

        /// Requests clients to hide the keyboard as soon as it's been used
        one_time_keyboard: bool,

        /// Use this parameter if you want to show the keyboard to specific users only
        selective: bool,

        /// The placeholder to be shown in the input field when the keyboard is active; 1-64 characters
        #[serde(skip_serializing_if = "Option::is_none")]
        input_field_placeholder: Option<String>,

        /// Requests clients to always show the keyboard in the chat (users may not otherwise see the keyboard)
        is_persistent: bool,
    },
    ReplyKeyboardRemove {
        /// Requests clients to remove the custom keyboard (user will not be
        /// able to summon this keyboard; if you want to hide the keyboard from
        /// sight but keep it accessible, use one_time_keyboard in ReplyKeyboardMarkup)
        remove_keyboard: bool,

        /// Use this parameter if you want to remove the keyboard for specific users only
        selective: bool,
    },
    ForceReply {
        /// Shows reply interface to the user, as if they manually selected the bot's message and tapped 'Reply'
        force_reply: bool,

        /// The placeholder to be shown in the input field when the keyboard is active; 1-64 characters
        #[serde(skip_serializing_if = "Option::is_none")]
        input_field_placeholder: Option<String>,

        /// Use this parameter if you want to force reply from specific users only
        selective: bool,
    },
}

impl ReplyMarkup {
    pub fn inline_keyboard_markup(inline_keyboard: Vec<Vec<InlineKeyboardButton>>) -> ReplyMarkup {
        ReplyMarkup::InlineKeyboardMarkup {
            inline_keyboard,
            resize_keyboard: false,
            one_time_keyboard: false,
            selective: false,
            input_field_placeholder: None,
            is_persistent: false,
        }
    }

    pub fn reply_keyboard_markup(keyboard: Vec<Vec<KeyboardButton>>) -> ReplyMarkup {
        ReplyMarkup::ReplyKeyboardMarkup {
            keyboard,
            resize_keyboard: false,
            one_time_keyboard: true,
            selective: false,
            input_field_placeholder: None,
            is_persistent: false,
        }
    }

    pub fn reply_keyboard_remove() -> ReplyMarkup {
        ReplyMarkup::ReplyKeyboardRemove {
            remove_keyboard: true,
            selective: false,
        }
    }

    pub fn force_reply() -> ReplyMarkup {
        ReplyMarkup::ForceReply {
            force_reply: true,
            input_field_placeholder: None,
            selective: false,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct SendMessageRequest {
    /// Unique identifier for the target chat or username of the target
    pub chat_id: i64,

    /// Text of the message to be sent
    pub text: String,

    /// If the message is a reply, ID of the original message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message_id: Option<i64>,

    /// Parse mode for the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,

    /// Reply markup for the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl Request for SendMessageRequest {}

impl SendMessageRequest {
    pub fn new(chat_id: i64, text: impl Into<String>) -> Self {
        Self {
            chat_id,
            text: text.into(),
            ..Default::default()
        }
    }

    pub fn with_reply_markup(mut self, reply_markup: ReplyMarkup) -> Self {
        self.reply_markup = Some(reply_markup);
        self
    }

    pub fn with_parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct EditMessageBase {
    /// Required if `inline_message_id` is not specified. Unique identifier for the
    /// target chat or username of the target channel (in the format @channelusername)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<i64>,

    /// Required if `inline_message_id` is not specified. Identifier of the message
    /// to edit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<i64>,

    /// Inline message identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,

    /// Mode for parsing entities in the message text. See formatting options for
    /// more details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,

    /// Reply markup for the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<String>,
}

impl EditMessageBase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_chat_id(mut self, chat_id: i64) -> Self {
        self.chat_id = Some(chat_id);
        self
    }

    pub fn with_message_id(mut self, message_id: i64) -> Self {
        self.message_id = Some(message_id);
        self
    }

    pub fn with_parse_mode(mut self, parse_mode: ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub fn with_reply_markup(mut self, reply_markup: ReplyMarkup) -> Self {
        self.reply_markup = Some(serde_json::to_string(&reply_markup).unwrap());
        self
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct EditMessageTextRequest {
    /// Base fields for edit requests
    #[serde(flatten)]
    pub base: EditMessageBase,

    /// The new text of the message, 1-4096 characters after entities parsing
    /// (Markdown or HTML)
    pub text: String,
}

impl EditMessageTextRequest {
    pub fn new(text: String) -> Self {
        Self {
            base: EditMessageBase::new(),
            text,
        }
    }

    pub fn with_chat_id(mut self, chat_id: i64) -> Self {
        self.base.chat_id = Some(chat_id);
        self
    }

    pub fn with_message_id(mut self, message_id: i64) -> Self {
        self.base.message_id = Some(message_id);
        self
    }
}

impl Request for EditMessageTextRequest {}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct EditMessageCaptionRequest {
    /// Base fields for edit requests
    #[serde(flatten)]
    pub base: EditMessageBase,

    /// New caption of the message, 0-1024 characters after entities parsing
    /// (Markdown or HTML)
    pub caption: String,
}

impl EditMessageCaptionRequest {
    pub fn new(caption: String) -> Self {
        Self {
            base: EditMessageBase::new(),
            caption,
        }
    }

    pub fn with_chat_id(mut self, chat_id: i64) -> Self {
        self.base.chat_id = Some(chat_id);
        self
    }
}

impl Request for EditMessageCaptionRequest {}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct EditMessageReplyMarkupRequest {
    /// Base fields for edit requests
    #[serde(flatten)]
    pub base: EditMessageBase,
}

impl EditMessageReplyMarkupRequest {
    pub fn new(reply_markup: ReplyMarkup) -> Self {
        Self {
            base: EditMessageBase::new().with_reply_markup(reply_markup),
        }
    }

    pub fn with_chat_id(mut self, chat_id: i64) -> Self {
        self.base.chat_id = Some(chat_id);
        self
    }

    pub fn with_message_id(mut self, message_id: i64) -> Self {
        self.base.message_id = Some(message_id);
        self
    }
}

impl Request for EditMessageReplyMarkupRequest {}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct DeleteMessageRequest {
    /// Unique identifier for the target chat or username of the target channel
    /// (in the format @channelusername)
    pub chat_id: i64,

    /// Identifier of the message to delete
    pub message_id: i64,
}

impl DeleteMessageRequest {
    pub fn new(chat_id: i64, message_id: i64) -> Self {
        Self {
            chat_id,
            message_id,
        }
    }
}

impl Request for DeleteMessageRequest {}

/// API methods for sending, editing, and deleting messages.
impl API {
    /// Send a message to a chat or channel.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use mobot::*;
    /// # #[tokio::main]
    /// # async fn main() {
    /// #    let api = API::new(Client::new(String::from("boo").into()));
    /// #    let chat_id = 123456789;
    ///    api.send_message(&api::SendMessageRequest::new(chat_id, "Hello!")).await;
    /// # }
    /// ```
    pub async fn send_message(&self, req: &SendMessageRequest) -> anyhow::Result<Message> {
        self.client.post("sendMessage", req).await
    }

    /// Edit the text of a previously sent message.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use mobot::*;
    /// # #[tokio::main]
    /// # async fn main() {
    /// #    let api = API::new(Client::new(String::from("boo").into()));
    /// #    let chat_id = 123456789;
    /// #    let message_id = 0;
    /// api.edit_message_text(
    ///   &api::EditMessageTextRequest::new(String::from("Changed my mind: Goodbye world!"))
    ///      .with_chat_id(chat_id)
    ///      .with_message_id(message_id)
    /// ).await;
    /// # }
    /// ```
    pub async fn edit_message_text(&self, req: &EditMessageTextRequest) -> anyhow::Result<Message> {
        self.client.post("editMessageText", req).await
    }

    /// Edit the caption of a message.
    pub async fn edit_message_caption(
        &self,
        req: &EditMessageCaptionRequest,
    ) -> anyhow::Result<Message> {
        self.client.post("editMessageCaption", req).await
    }

    /// Edit the reply markup of a message.
    pub async fn edit_message_reply_markup(
        &self,
        req: &EditMessageReplyMarkupRequest,
    ) -> anyhow::Result<Message> {
        self.client.post("editMessageReplyMarkup", req).await
    }

    /// Delete a message.
    pub async fn delete_message(&self, req: &DeleteMessageRequest) -> anyhow::Result<()> {
        self.client.post("deleteMessage", req).await
    }

    pub async fn remove_reply_keyboard(
        &self,
        chat_id: i64,
        text: String,
    ) -> anyhow::Result<Message> {
        self.send_message(
            &SendMessageRequest::new(chat_id, text)
                .with_reply_markup(ReplyMarkup::reply_keyboard_remove()),
        )
        .await
    }
}
