use anyhow::Result;
use derive_more::*;

use crate::update::GetUpdatesResponse;

#[derive(Debug, Clone, From, Into, FromStr, Display)]
pub struct ApiToken(String);

#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
}

impl Client {
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

    pub async fn get_updates(&self) -> Result<GetUpdatesResponse> {
        let body = reqwest::get(format!("{}/getUpdates", self.base_url))
            .await?
            .text()
            .await?;

        println!("body = {body}");

        let updates: GetUpdatesResponse = serde_json::from_str(&body)?;
        Ok(updates)
    }
}
