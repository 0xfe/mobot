use anyhow::Result;
use serde::Deserialize;
use thiserror::Error;

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
#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse<T> {
    /// `true` if the request was successful.
    pub ok: bool,

    /// Error description, if `ok` is `false`.
    pub description: Option<String>,

    /// The result of the request, if `ok` is `true`.
    pub result: Option<T>,
}

impl<T> ApiResponse<T> {
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
