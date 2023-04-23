use derive_more::{From, Into};
use serde::{Deserialize, Serialize};

use crate::{ApiResponse, Message};

#[derive(Debug, Clone, Deserialize)]
pub struct Update {
    /// The update‘s unique identifier. Update identifiers start from a
    /// certain positive number and increase sequentially. This ID becomes
    /// especially handy if you’re using Webhooks, since it allows you to
    /// ignore repeated updates or to restore the correct update sequence,
    /// should they get out of order. If there are no new updates for at
    /// least a week, then identifier of the next update will be chosen
    /// randomly instead of sequentially.
    pub update_id: i64,

    pub message: Option<Message>,

    pub edited_message: Option<Message>,

    pub channel_post: Option<Message>,

    pub edited_channel_post: Option<Message>,
}

/// Use this method to receive incoming updates using long or short
/// polling. An Array of Update objects is returned.
#[derive(Debug, Clone, Serialize, Default)]
pub struct GetUpdatesRequest {
    pub offset: Option<i64>,

    /// Limits the number of updates to be retrieved. Defaults to 100.
    pub limit: Option<i64>,

    /// Timeout in seconds for long polling.
    pub timeout: Option<i64>,
    pub allowed_updates: Option<Vec<String>>,
}

impl GetUpdatesRequest {
    pub fn new() -> Self {
        Self::default()
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

#[derive(Debug, Clone, Deserialize, From, Into)]
pub struct GetUpdatesResponse(ApiResponse<Vec<Update>>);
