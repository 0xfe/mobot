use mobot_derive::BotRequest;
use serde::{Deserialize, Serialize};

use super::{user::User, API};

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct CallbackQuery {
    /// Unique identifier for this query
    pub id: String,

    /// Sender of the query
    pub from: User,

    /// Message with the callback button that originated the query. Note that
    /// message content and message date will not be available if the message
    /// is too old.
    pub message: Option<super::Message>,

    /// Identifier of the message sent via the bot in inline mode, that
    /// originated the query.
    pub inline_message_id: Option<String>,

    /// Data associated with the callback button. Be aware that a bad client
    /// can send arbitrary data in this field.
    pub data: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InlineQuery {
    /// Unique identifier for this query
    pub id: String,

    /// Sender
    pub from: User,

    /// Text of the query (up to 512 characters)
    pub query: String,

    /// Offset of the results to be returned, can be controlled by the bot
    pub offset: String,
}

#[derive(Debug, Serialize, Clone, Default, BotRequest)]
pub struct AnswerInlineQuery {
    /// Unique identifier for the answered query
    pub inline_query_id: String,

    /// A JSON-serialized array of results for the inline query
    pub results: Vec<InlineQueryResultArticle>,

    /// The maximum amount of time in seconds that the result of the inline query
    /// may be cached on the server. Defaults to 300.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_time: Option<i64>,

    /// Pass True, if results may be cached on the server side only for the user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_personal: Option<bool>,

    /// Pass the offset that a client should send in the next query with the same
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<String>,
}

impl AnswerInlineQuery {
    pub fn new(inline_query_id: String) -> Self {
        Self {
            inline_query_id,
            ..Default::default()
        }
    }

    pub fn with_article_text(self, title: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            inline_query_id: self.inline_query_id,
            results: vec![InlineQueryResultArticle {
                id: "0".to_string(),
                result_type: "article".to_string(),
                title: title.into(),
                input_message_content: InputMessageContent {
                    message_text: text.into(),
                },
            }],
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct InlineQueryResultArticle {
    /// Unique identifier for this result, 1-64 Bytes
    pub id: String,

    /// Type of the result
    #[serde(rename = "type")]
    pub result_type: String,

    /// Title of the result
    pub title: String,

    /// Content of the message to be sent
    pub input_message_content: InputMessageContent,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct InputMessageContent {
    /// Text of the message to be sent, 1-4096 characters
    pub message_text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, BotRequest)]
pub struct AnswerCallbackQueryRequest {
    /// Unique identifier for the query to be answered
    pub callback_query_id: String,

    /// Text of the notification. If not specified, nothing will be shown to the user, 0-200 characters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// If true, an alert will be shown by the client instead of a notification at the top of the chat screen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_alert: Option<bool>,

    /// URL that will be opened by the user's client. If you have created a Game and accepted the conditions via @Botfather,
    /// specify the URL that opens your game – note that this will only work if the query comes from a callback_game button.
    /// Otherwise, you may use links like telegram.me/your_bot?start=XXXX that open your bot with a parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// The maximum amount of time in seconds that the result of the callback query may be cached client-side.
    /// Telegram apps will support caching starting in version 3.14. Defaults to 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_time: Option<i64>,
}

impl AnswerCallbackQueryRequest {
    pub fn new(callback_query_id: String) -> Self {
        Self {
            callback_query_id,
            ..Default::default()
        }
    }

    pub fn with_text(self, text: impl Into<String>) -> Self {
        Self {
            callback_query_id: self.callback_query_id,
            text: Some(text.into()),
            ..Default::default()
        }
    }

    pub fn with_show_alert(self, show_alert: bool) -> Self {
        Self {
            callback_query_id: self.callback_query_id,
            show_alert: Some(show_alert),
            ..Default::default()
        }
    }
}

impl API {
    pub async fn answer_inline_query(&self, req: &AnswerInlineQuery) -> anyhow::Result<bool> {
        self.client.post("answerInlineQuery", req).await
    }

    pub async fn answer_callback_query(
        &self,
        req: &AnswerCallbackQueryRequest,
    ) -> anyhow::Result<bool> {
        self.client.post("answerCallbackQuery", req).await
    }
}
