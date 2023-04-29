use serde::{Deserialize, Serialize};

use crate::{Request, TelegramClient, User, API};

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

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
pub struct InputMessageContent {
    /// Text of the message to be sent, 1-4096 characters
    pub message_text: String,
}

impl Request for AnswerInlineQuery {}

impl<T: TelegramClient> API<T> {
    pub async fn answer_inline_query(&self, req: &AnswerInlineQuery) -> anyhow::Result<bool> {
        self.client.post("answerInlineQuery", req).await
    }
}
