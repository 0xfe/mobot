use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Handler error: {0}")]
    Failed(String),
}

impl<T: Into<String>> From<T> for Error {
    fn from(s: T) -> Self {
        Error::Failed(s.into())
    }
}

#[derive(Debug, Clone)]
pub enum Action<T> {
    Next(T),
    Done(T),
}
