/// `Action` represents an action to take after handling a chat event.
#[derive(Debug, Clone)]
pub enum Action {
    /// Continue to the next handler.
    Next,

    /// Stop handling events.
    Done,

    /// Reply to the message with the given text and stop handling events. This
    /// is equivalent to `e.send_text(...)` followed by `Ok(Action::Done)`.
    ReplyText(String),

    /// Same as ReplyText, but with MarkdownV2 formatting. Make
    /// sure to escape any user input!
    ReplyMarkdown(String),

    /// Reply to the message with the given sticker and stop running handlers.
    ReplySticker(String),
}
