use mobot_derive::BotRequest;
use serde::{Deserialize, Serialize};

use super::API;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    /// Identifier for this file, which can be used to download or reuse the file
    pub file_id: String,

    /// File size
    pub file_size: Option<i64>,

    /// File path. Use mobot::api::get_file to get the file
    pub file_path: Option<String>,
}

#[derive(Debug, Serialize, Clone, BotRequest)]
pub struct GetFileRequest {
    /// Unique identifier for target file
    pub file_id: String,
}

impl GetFileRequest {
    pub fn new(file_id: String) -> Self {
        Self { file_id }
    }
}

impl API {
    pub async fn get_file(&self, req: &GetFileRequest) -> anyhow::Result<File> {
        self.client.post("getFile", req).await
    }
}
