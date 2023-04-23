use anyhow::Result;
use derive_more::*;

use crate::{
    update::{GetUpdatesRequest, Update},
    ApiResponse,
};

#[derive(Debug, Clone, From, Into, FromStr, Display)]
pub struct ApiToken(String);

/// This is the main Telegram API client. Requires a valid API token.
#[derive(Debug, Clone)]
pub struct Client {
    /// This base URL is used for all requests and is constructed from the
    /// provided API token.
    base_url: String,
}

impl Client {
    /// Returns a new Telegram API client.
    pub fn new(token: ApiToken) -> Self {
        Self {
            base_url: format!("https://api.telegram.org/bot{token}"),
        }
    }

    pub async fn get_me(&self) -> Result<()> {
        let body = reqwest::get(format!("{}/getMe", self.base_url))
            .await?
            .text()
            .await?;

        println!("body = {body}");
        Ok(())
    }

    pub async fn get_updates(&self, req: GetUpdatesRequest) -> Result<Vec<Update>> {
        let client = reqwest::Client::new();

        let response = client
            .post(format!("{}/getUpdates", self.base_url))
            .json(&req)
            .send()
            .await?
            .text()
            .await?;

        debug!("response(get_updates) = {response}");

        let updates: ApiResponse<Vec<Update>> = serde_json::from_str(&response)?;
        Ok(updates.result()?.clone())
    }
}
