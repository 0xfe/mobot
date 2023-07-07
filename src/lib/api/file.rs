use bytes;
use mobot_derive::BotRequest;
use serde::{Deserialize, Serialize};

use super::API;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    /// Identifier for this file, which can be used to download or reuse the file
    pub file_id: String,

    /// File size
    pub file_size: Option<i64>,

    /// File path. You can use it with api.download_file to download file
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

#[derive(Debug, Serialize, Clone, BotRequest)]
pub struct DownloadRequest {
    /// File path. You can use it with api.download_file to download file
    pub file_path: String,
}

impl DownloadRequest {
    pub fn new(file_path: String) -> Self {
        Self { file_path }
    }
}

impl API {
    pub async fn get_file(&self, req: &GetFileRequest) -> anyhow::Result<File> {
        self.client.post("getFile", req).await
    }
    /// Download file by its file_path. File must be no larger than 20 mb
    pub async fn download_file(&self, req: &DownloadRequest) -> anyhow::Result<bytes::Bytes> {
        self.client.download_file(&req.file_path).await
    }
}
