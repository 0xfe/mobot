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

#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse<T> {
    pub ok: bool,
    pub description: Option<String>,
    pub result: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn is_ok(&self) -> bool {
        self.ok
    }

    pub fn result(&self) -> Result<&T> {
        if !self.ok {
            return Err(ApiError::AppError(self.description.clone().unwrap()).into());
        }

        if self.result.is_none() {
            return Err(ApiError::NoResult.into());
        }

        Ok(self.result.as_ref().unwrap())
    }
}
