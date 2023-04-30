use serde::{Deserialize, Serialize};

use crate::{Request, TelegramClient, API};

use super::user::User;

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Serialize, Clone, Default)]
pub struct AnswerInlineQuery {
    /// Unique identifier for the answered query
    pub inline_query_id: String,

    /// A JSON-serialized array of results for the inline query
    pub results: Vec<InlineQueryResultArticle>,

    /// The maximum amount of time in seconds that the result of the inline query
    /// may be cached on the server. Defaults to 300.
    pub cache_time: Option<i64>,

    /// Pass True, if results may be cached on the server side only for the user
    pub is_personal: Option<bool>,

    /// Pass the offset that a client should send in the next query with the same
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

impl Request for AnswerInlineQuery {}

impl API {
    pub async fn answer_inline_query(&self, req: &AnswerInlineQuery) -> anyhow::Result<bool> {
        self.client.post("answerInlineQuery", req).await
    }
}
