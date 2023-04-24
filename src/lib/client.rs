use anyhow::Result;
use derive_more::*;
use serde::de::DeserializeOwned;

use crate::{
    update::{GetUpdatesRequest, Update},
    ApiResponse, Event, Message, SendStickerRequest,
};

#[derive(Debug, Clone, From, Into, FromStr, Display)]
pub struct ApiToken(String);

/// This is the main Telegram API client. Requires a valid API token.
#[derive(Debug, Clone)]
pub struct Client {
    /// This base URL is used for all requests and is constructed from the
    /// provided API token.
    base_url: String,
    client: reqwest::Client,
}

impl Client {
    /// Returns a new Telegram API client.
    pub fn new(token: ApiToken) -> Self {
        Self {
            base_url: format!("https://api.telegram.org/bot{token}"),
            client: reqwest::Client::new(),
        }
    }

    pub async fn post<Req, Resp>(&self, method: &str, req: &Req) -> Result<Resp>
    where
        Req: crate::Request,
        Resp: DeserializeOwned + Clone,
    {
        let body = self
            .client
            .post(format!("{}/{}", self.base_url, method))
            .json(&req)
            .send()
            .await?
            .text()
            .await?;

        let response = ApiResponse::<Resp>::from_str(&body)?;
        Ok(response.result()?.clone())
    }

    pub async fn get_me(&self) -> Result<()> {
        let body = reqwest::get(format!("{}/getMe", self.base_url))
            .await?
            .text()
            .await?;

        println!("body = {body}");
        Ok(())
    }

    pub async fn get_updates(&self, req: &GetUpdatesRequest) -> Result<Vec<Update>> {
        self.post("getUpdates", req).await
    }

    pub async fn get_events(&self, req: &GetUpdatesRequest) -> Result<Vec<Event>> {
        Ok(self
            .get_updates(req)
            .await?
            .into_iter()
            .map(|u| u.into())
            .collect())
    }

    pub async fn send_sticker(&self, req: &SendStickerRequest) -> Result<Message> {
        self.post("sendSticker", req).await
    }
}
