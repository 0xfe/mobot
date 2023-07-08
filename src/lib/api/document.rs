use serde::{Deserialize, Serialize};

use super::PhotoSize;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Document {
    /// Identifier for this file, which can be used to download or reuse the file
    pub file_id: String,

    /// Document thumbnail as defined by sender
    pub thumbnail: Option<PhotoSize>,

    /// Original filename as defined by sender
    pub file_name: Option<String>,

    /// MIME type of the file as defined by sender
    pub mime_type: Option<String>,

    /// File size
    pub file_size: Option<i64>,
}
