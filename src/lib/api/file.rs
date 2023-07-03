use mobot_derive::BotRequest;
use serde::{Deserialize, Serialize};

use super::{API};

#[derive(Debug, Clone, Deserialize, Serialize, BotRequest)]
pub struct File {
    /// Unique identifier for this file
    pub file_id: String,

    /// File size
    pub file_size: Option<i64>,

    pub file_path: Option<String>
}

#[derive(Debug, Serialize, Clone, BotRequest)]
pub struct GetFileRequest {
    /// Unique identifier for target file
    pub file_id: String,
}

impl GetFileRequest {
    pub fn new(file_id: String) -> Self {
        Self {
            file_id
        }
    }
}

impl API {
    pub async fn get_file(&self, req: &GetFileRequest) -> anyhow::Result<File> {
        self.client.post("getFile", req).await
    }
}