use serde::{Deserialize, Serialize};

use crate::{Matcher, Request, Route, API};

use super::{message::Message, query::InlineQuery, CallbackQuery};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Update {
    /// The update‘s unique identifier. Update identifiers start from a
    /// certain positive number and increase sequentially. This ID becomes
    /// especially handy if you’re using Webhooks, since it allows you to
    /// ignore repeated updates or to restore the correct update sequence,
    /// should they get out of order. If there are no new updates for at
    /// least a week, then identifier of the next update will be chosen
    /// randomly instead of sequentially.
    pub update_id: i64,

    /// New incoming message of any kind — text, photo, sticker, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,

    /// New version of a message that is known to the bot and was edited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_message: Option<Message>,

    /// New incoming channel post of any kind — text, photo, sticker, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_post: Option<Message>,

    /// New version of a channel post that is known to the bot and was
    /// edited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_channel_post: Option<Message>,

    /// New incoming inline query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_query: Option<InlineQuery>,

    /// Callbakc query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_query: Option<CallbackQuery>,
}

impl Update {
    pub fn parts(&self) -> anyhow::Result<(i64, Route)> {
        if let Some(ref m) = self.message {
            debug!("New message: {:#?}", m);
            Ok((m.chat.id, Route::NewMessage(Matcher::Any)))
        } else if let Some(ref m) = self.edited_message {
            debug!("Edited message: {:#?}", m);
            Ok((m.chat.id, Route::EditedMessage(Matcher::Any)))
        } else if let Some(ref m) = self.channel_post {
            debug!("Channel post: {:#?}", m);
            Ok((m.chat.id, Route::ChannelPost(Matcher::Any)))
        } else if let Some(ref m) = self.edited_channel_post {
            debug!("Edited channel post: {:#?}", m);
            Ok((m.chat.id, Route::EditedChannelPost(Matcher::Any)))
        } else if let Some(ref q) = self.callback_query {
            debug!("Callback query: {:#?}", q);
            Ok((
                q.message.as_ref().map(|m| m.chat.id).unwrap_or(0),
                Route::CallbackQuery(Matcher::Any),
            ))
        } else {
            Err(anyhow::anyhow!("Unknown update type"))
        }
    }
}

/// Use this method to receive incoming updates using long or short
/// polling. An Array of Update objects is returned.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GetUpdatesRequest {
    /// Identifier of the first update to be returned. Must be greater by
    /// one than the highest among the identifiers of previously received
    /// updates. By default, updates starting with the earliest unconfirmed
    /// update are returned. An update is considered confirmed as soon as
    /// getUpdates is called with an offset higher than its update_id. The
    /// negative offset can be specified to retrieve updates starting from
    /// -offset update from the end of the updates queue. All previous
    /// updates will forgotten.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,

    /// Limits the number of updates to be retrieved. Defaults to 100.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// Timeout in seconds for long polling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i64>,

    /// List the types of updates you want your bot to receive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_updates: Option<Vec<String>>,
}

impl Request for GetUpdatesRequest {}

/// Convenience methods for `GetUpdatesRequest`.
impl GetUpdatesRequest {
    pub fn new() -> Self {
        Self {
            allowed_updates: Some(vec![]),
            ..Default::default()
        }
    }

    pub fn with_limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_timeout(mut self, timeout: i64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }
}

impl API {
    /// Use this method to receive incoming updates using long polling. An
    /// Array of Update objects is returned. See [the official docs](https://core.telegram.org/bots/api#getupdates)
    /// for more information.
    pub async fn get_updates(&self, req: &GetUpdatesRequest) -> anyhow::Result<Vec<Update>> {
        self.client.post("getUpdates", req).await
    }
}
