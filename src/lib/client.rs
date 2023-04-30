use std::fmt::{self, Formatter};

use anyhow::Result;
use derive_more::*;
use serde::{de::DeserializeOwned, Serialize};

use crate::ApiResponse;

/// This is a wrapper around the Telegram API token string. Get your token from
/// [@BotFather][1].
#[derive(Debug, Clone, From, Into, FromStr, Display)]
pub struct ApiToken(String);

/// PostFn lets you override the default POST request handler. This is useful
/// for testing.
pub struct PostFn(pub Box<dyn Fn(String, String) -> Result<String> + Send + Sync>);

/// From trait for PostFn to make it easier to use.
impl<T> From<T> for PostFn
where
    T: Fn(String, String) -> Result<String> + Send + Sync + 'static,
{
    fn from(f: T) -> Self {
        Self(Box::new(f))
    }
}

/// Debug trait for PostFn (because Client derives Debug)
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

    /// The underlying HTTP client.
    client: reqwest::Client,

    /// A function that handles POST requests. This is useful for testing.
    post_handler: Option<PostFn>,
}

impl Client {
    /// Returns a new Telegram API client.
    pub fn new(token: ApiToken) -> Self {
        Self {
            base_url: format!("https://api.telegram.org/bot{token}"),
            client: reqwest::Client::new(),
            post_handler: None,
        }
    }

    /// Sets a function that handles POST requests. This is useful for testing.
    pub fn with_post_handler(mut self, post_fn: impl Into<PostFn>) -> Self {
        self.post_handler = Some(post_fn.into());
        self
    }

    /// Send `method` with `req` as the request body to the Telegram API.
    pub async fn post<Req, Resp>(&self, method: &str, req: &Req) -> Result<Resp>
    where
        Req: crate::Request,
        Resp: Serialize + DeserializeOwned + Clone,
    {
        let body;
        if let Some(ref post_handler) = self.post_handler {
            body = (post_handler.0)(method.to_string(), serde_json::to_string(req)?).unwrap();
        } else {
            debug!(
                "POST /{}:\n{}",
                method,
                serde_json::to_string_pretty(req).unwrap()
            );
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
        debug!(
            "Response /{}:\n{}",
            method,
            serde_json::to_string_pretty(&response).unwrap()
        );
        Ok(response.result()?.clone())
    }
}
