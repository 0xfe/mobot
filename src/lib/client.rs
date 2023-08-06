use std::fmt::{self, Formatter};

use anyhow::Result;
use bytes;
use derive_more::*;
use serde::{de::DeserializeOwned, Serialize};

use crate::api::ApiResponse;

/// This is a wrapper around the Telegram API token string. Get your token from
/// [@BotFather](https://t.me/BotFather).
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

#[async_trait::async_trait]
pub trait Post {
    async fn post(&self, method: String, req: String) -> Result<String>;
}

/// This is a thin shim around the Telegram HTTP client. Requires a valid API token.
pub struct Client {
    /// This base URL is used for all requests and is constructed from the
    /// provided API token.
    base_url: String,

    /// This is URL is used for requests for download files
    file_url: String,

    /// The underlying HTTP client.
    client: reqwest::Client,

    /// A post handler that implements the Post trait. Useful for testing.
    post_handler: Option<Box<dyn Post + Send + Sync>>,

    /// A function that handles POST requests. This is useful for testing.
    post_handler_fn: Option<PostFn>,
}

impl Client {
    /// Returns a new Telegram API client.
    pub fn new(token: impl Into<ApiToken>) -> Self {
        let token = token.into();
        Self {
            base_url: format!("https://api.telegram.org/bot{token}"),
            file_url: format!("https://api.telegram.org/file/bot{token}"),
            client: reqwest::Client::new(),
            post_handler: None,
            post_handler_fn: None,
        }
    }

    /// Sets a function that handles POST requests. This is useful for testing.
    pub fn with_post_handler_fn(mut self, post_fn: impl Into<PostFn>) -> Self {
        self.post_handler_fn = Some(post_fn.into());
        self
    }

    pub fn with_post_handler(mut self, post_handler: impl Post + Send + Sync + 'static) -> Self {
        self.post_handler = Some(Box::new(post_handler));
        self
    }

    /// Send `method` with `req` as the request body to the Telegram API.
    pub async fn post<Req, Resp>(&self, method: &str, req: &Req) -> Result<Resp>
    where
        Req: crate::api::Request,
        Resp: Serialize + DeserializeOwned + Clone,
    {
        let body;
        if let Some(ref post_handler) = self.post_handler_fn {
            body = (post_handler.0)(method.to_string(), serde_json::to_string(req)?).unwrap();
        } else if let Some(ref post_handler) = self.post_handler {
            body = post_handler
                .post(method.to_string(), serde_json::to_string(req)?)
                .await?;
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

    pub async fn download_file(&self, file_path: &String) -> Result<bytes::Bytes> {
        debug!("Downloading file /{}:\n", file_path);
        let body = self
            .client
            .get(format!("{}/{}", self.file_url, file_path))
            .send()
            .await?
            .bytes()
            .await?;
        debug!("File downloaded successfully /{}:\n", file_path,);
        Ok(body)
    }
}
