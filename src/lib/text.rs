use derive_more::Display;

use crate::api;

/// `Text` is a wrapper around `String` that allows you to specify the parse mode for
/// Telegram messages.
#[derive(Clone, Debug, Display)]
pub enum Text {
    Plain(String),
    Markdown(String),
}

impl From<Text> for String {
    fn from(text: Text) -> Self {
        match text {
            Text::Plain(text) => text,
            Text::Markdown(text) => text,
        }
    }
}

impl From<Text> for api::ParseMode {
    fn from(text: Text) -> Self {
        match text {
            Text::Plain(_) => api::ParseMode::Text,
            Text::Markdown(_) => api::ParseMode::MarkdownV2,
        }
    }
}

impl From<&str> for Text {
    fn from(text: &str) -> Self {
        Text::Plain(text.into())
    }
}

impl From<String> for Text {
    fn from(text: String) -> Self {
        Text::Plain(text)
    }
}
