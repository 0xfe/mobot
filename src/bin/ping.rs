#![allow(dead_code)]

#[macro_use]
extern crate log;

use std::{collections::HashMap, env, sync::Arc};

use anyhow::Context;
use mogram::{router::*, Client, SendStickerRequest};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct ChatState {
    stickers: Arc<Vec<&'static str>>,
    counters: Arc<Mutex<HashMap<i64, usize>>>,
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
            stickers: Arc::new(stickers),
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

async fn handle_chat_event(e: ChatEvent, state: ChatState) -> Result<Action<ChatAction>, Error> {
    match e.message {
        MessageEvent::New(message) => {
            let api = e.api.read().await;
            let mut counters = state.counters.lock().await;
            let counter = counters.entry(message.chat.id).or_insert(0);
            *counter += 1;

            api.send_sticker(&SendStickerRequest::new(
                message.chat.id,
                state
                    .stickers
                    .get(*counter % state.stickers.len())
                    .unwrap()
                    .to_string(),
            ))
            .await
            .context("sendSticker")
            .or(Err(Error::Failed("terrible".to_string())))?;

            Ok(Action::Next(ChatAction::ReplyText(format!(
                "pong({}): {}",
                counter,
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

    router.add_chat_handler(ChatHandler::new(handle_chat_event).with_state(ChatState::default()));

    router.start().await;
}
