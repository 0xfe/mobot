use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::Client;

/// This is the main Telegram API client. Requires an instance of `Client` initialized
/// with a valid API token.
pub struct API {
    /// The underlying HTTP client.
    pub client: Client,
}

impl API {
    /// Returns a new Telegram API client.
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

/// Request is a trait that all Telegram API requests must implement.
pub trait Request: Serialize + Send + Sync {}

/// APIError wraps error messages returned by the Telegram API.
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Telegram error: {0}")]
    AppError(String),

    #[error("Client error: {0}")]
    ClientError(String),

    #[error("No result")]
    NoResult,
}

/// This is a wrapper around the Telegram API response. If `ok` is `true`, then
/// `result` is guaranteed to be `Some`. If `ok` is `false`, then `description`
/// is guaranteed to be `Some`, with a description of the error.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    /// `true` if the request was successful.
    pub ok: bool,

    /// Error description, if `ok` is `false`.
    pub description: Option<String>,

    /// The result of the request, if `ok` is `true`.
    pub result: Option<T>,
}

#[allow(clippy::should_implement_trait)]
impl<'de, T: Deserialize<'de>> ApiResponse<T> {
    pub fn from_str(data: &'de str) -> Result<Self> {
        let response: ApiResponse<T> = serde_json::from_str(data)?;
        Ok(response)
    }
}

impl<T> ApiResponse<T> {
    /// Wraps the result in an `Ok` ApiResponse.
    #[allow(non_snake_case)]
    pub fn Ok(result: T) -> Self {
        Self {
            ok: true,
            description: None,
            result: Some(result),
        }
    }

    /// Creates an error response with the given description.
    #[allow(non_snake_case)]
    pub fn Err(description: impl Into<String>) -> Self {
        Self {
            ok: false,
            description: Some(description.into()),
            result: None,
        }
    }

    /// Returns `true` if the request was successful.
    pub fn is_ok(&self) -> bool {
        self.ok
    }

    /// Returns the result of the request, if `ok` is `true`. Otherwise, returns
    /// an error.
    pub fn result(&self) -> Result<&T> {
        if !self.ok {
            return Err(ApiError::AppError(
                self.description
                    .clone()
                    .unwrap_or("No error description".to_string()),
            )
            .into());
        }

        if self.result.is_none() {
            return Err(ApiError::NoResult.into());
        }

        Ok(self.result.as_ref().unwrap())
    }
}
