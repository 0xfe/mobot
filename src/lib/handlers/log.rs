use crate::{Action, Event, Update};

/// This handler logs every message received.
pub async fn log_handler<S>(e: Event, _: S) -> Result<Action, anyhow::Error> {
    match e.update {
        Update::Message(message)
        | Update::EditedMessage(message)
        | Update::ChannelPost(message)
        | Update::EditedChannelPost(message) => {
            let chat_id = message.chat.id;
            let from = message.from.unwrap_or_default();
            let text = message.text.unwrap_or_default();

            info!("({}) Message from {}: {}", chat_id, from.first_name, text);

            Ok(Action::Next)
        }
        Update::CallbackQuery(query) => {
            let chat_id = query.message.unwrap_or_default().chat.id;
            let from = query.from;
            let data = query.data.unwrap_or_default();

            info!("({}) Callback from {}: {}", chat_id, from.first_name, data);

            Ok(Action::Next)
        }
        _ => Err(anyhow::anyhow!("Unknown message type")),
    }
}
