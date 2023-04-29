use std::fmt::{self, Formatter};

use anyhow::Result;
use async_trait::async_trait;
use derive_more::*;
use serde::de::DeserializeOwned;

use crate::ApiResponse;

#[async_trait]
pub trait TelegramClient: Send + Sync {
    async fn post<Req, Resp>(&self, method: &str, req: &Req) -> Result<Resp>
    where
        Req: crate::Request,
        Resp: DeserializeOwned + Clone;
}

#[derive(Debug, Clone, From, Into, FromStr, Display)]
pub struct ApiToken(String);

pub struct PostFn(pub Box<dyn Fn(String, String) -> Result<String> + Send + Sync>);

impl fmt::Debug for PostFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("PostFn")
    }
}

/// This is the main Telegram API client. Requires a valid API token.
#[derive(Debug)]
pub struct Client {
    /// This base URL is used for all requests and is constructed from the
    /// provided API token.
    base_url: String,
    client: reqwest::Client,
    post_fn: Option<PostFn>,
}

impl Client {
    /// Returns a new Telegram API client.
    pub fn new(token: ApiToken) -> Self {
        Self {
            base_url: format!("https://api.telegram.org/bot{token}"),
            client: reqwest::Client::new(),
            post_fn: None,
        }
    }

    pub fn set_post_fn(&mut self, post_fn: PostFn) {
        self.post_fn = Some(post_fn);
    }

    pub async fn get_me(&self) -> Result<()> {
        let body = reqwest::get(format!("{}/getMe", self.base_url))
            .await?
            .text()
            .await?;

        println!("body = {body}");
        Ok(())
    }
}

#[async_trait]
impl TelegramClient for Client {
    async fn post<Req, Resp>(&self, method: &str, req: &Req) -> Result<Resp>
    where
        Req: crate::Request,
        Resp: DeserializeOwned + Clone,
    {
        let body;
        if let Some(ref post_fn) = self.post_fn {
            body = (post_fn.0)(method.to_string(), serde_json::to_string(req)?).unwrap();
        } else {
            body = self
                .client
                .post(format!("{}/{}", self.base_url, method))
                .json(&req)
                .send()
                .await?
                .text()
                .await?;
        }

        let response = ApiResponse::<Resp>::from_str(&body)?;
        Ok(response.result()?.clone())
    }
}
