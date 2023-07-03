use mobot_derive::BotRequest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, BotRequest)]
pub struct PhotoSize {
    /// Unique identifier for this file
    pub file_id: String,

    /// Sticker width
    pub width: i64,

    /// Sticker height
    pub height: i64,

    /// File size
    pub file_size: Option<i64>,
}