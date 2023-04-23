use derive_more::{From, Into};
use serde::Deserialize;

use crate::ApiResponse;

#[derive(Debug, Clone, Deserialize)]
pub struct Update {
    #[serde(default)]
    pub update_id: i64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GetUpdatesRequest {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub timeout: Option<i64>,
    pub allowed_updates: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, From, Into)]
pub struct GetUpdatesResponse(ApiResponse<Vec<Update>>);
