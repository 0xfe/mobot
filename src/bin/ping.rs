/// This is a simple ping bot that responds to every message with a sticker and
/// a text message.

#[macro_use]
extern crate log;

use std::{env, sync::Arc};

use anyhow::Context;
use mogram::{router::*, Client, SendStickerRequest, TelegramClient};
use tokio::sync::Mutex;

/// The state of the chat. This is a simple counter that is incremented every
/// time a message is received.
#[derive(Debug, Clone)]
struct ChatState {
    stickers: Vec<&'static str>,
    counter: usize,
}

impl Default for ChatState {
    fn default() -> Self {
        let stickers: Vec<&str> = vec![
            "CAACAgIAAxkBAAEgIVZkRIhFH8RwQ-rH2mAPiT5JyrniMwACqBYAAthnYUhcJMSWynpYVi8E",
            "CAACAgIAAxkBAAEgIU5kRIeMS9WRRu4E8IOpqXf3YrphYQACSgcAAkb7rAQjQpsNX97E4C8E",
            "CAACAgIAAxkBAAEgIVpkRIihHTdCZpqTxIyzNW2Is2LzFQACcAoAAlxVeUsylSm19qsaAAEvBA",
            "CAACAgIAAxkBAAEgIV5kRIjd-5ysEwPs4Npl8RJNVIfjLAACIw4AAp9bSEq43bNL_8rWFi8E",
            "CAACAgIAAxkBAAEgIWJkRIj-MeWwv364OtXrcsTClGue9AACcg8AAibMiUpsrVGWrFXUvS8E",
            "CAACAgIAAxkBAAEgIWRkRIkbtjVgTqOPklYgo9Vo4Y2_1wACRQsAAsh2GUteeO5PO-ys-y8E",
            "CAACAgIAAxkBAAEgIWhkRIlET04f4SaUmVF2LdU2hZG-EgACzhQAAqOI2Ep-yBn6va_C5C8E",
        ];

        Self {
            stickers,
            counter: 0,
        }
    }
}

/// The handler for the chat. This is a simple function that takes a `ChatEvent`
/// and returns a `ChatAction`.
async fn handle_chat_event<T: TelegramClient>(
    e: ChatEvent<T>,
    state: Arc<Mutex<ChatState>>,
) -> Result<Action<ChatAction>, Error> {
    let mut state = state.lock().await;

    match e.message {
        MessageEvent::New(message) => {
            state.counter += 1;

            e.api
                .send_sticker(&SendStickerRequest::new(
                    message.chat.id,
                    state
                        .stickers
                        .get(state.counter % state.stickers.len())
                        .unwrap()
                        .to_string(),
                ))
                .await
                .context("sendSticker")
                .or(Err(Error::Failed("terrible".to_string())))?;

            Ok(Action::Next(ChatAction::ReplyText(format!(
                "pong({}): {}",
                state.counter,
                message.text.unwrap_or_default()
            ))))
        }
        _ => Err(Error::Failed("Unhandled update".into())),
    }
}

#[tokio::main]
async fn main() {
    mogram::init_logger();
    info!("Starting pingbot...");

    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap().into());
    let mut router = Router::new(client);

    router.add_chat_handler(ChatHandler::new(handle_chat_event));
    router.start().await;
}
