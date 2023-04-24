#![allow(dead_code)]

#[macro_use]
extern crate log;

use std::{cmp::max, collections::HashMap, env};

use mogram::{update::GetUpdatesRequest, Client, Event, SendStickerRequest, API};

#[derive(Debug, Clone, Default)]
struct Chats {
    counters: HashMap<i64, usize>,
}

#[tokio::main]
async fn main() {
    mogram::init_logger();
    info!("Starting mobot...");

    let stickers: Vec<&str> = vec![
        "CAACAgIAAxkBAAEgIVZkRIhFH8RwQ-rH2mAPiT5JyrniMwACqBYAAthnYUhcJMSWynpYVi8E",
        "CAACAgIAAxkBAAEgIU5kRIeMS9WRRu4E8IOpqXf3YrphYQACSgcAAkb7rAQjQpsNX97E4C8E",
        "CAACAgIAAxkBAAEgIVpkRIihHTdCZpqTxIyzNW2Is2LzFQACcAoAAlxVeUsylSm19qsaAAEvBA",
        "CAACAgIAAxkBAAEgIV5kRIjd-5ysEwPs4Npl8RJNVIfjLAACIw4AAp9bSEq43bNL_8rWFi8E",
        "CAACAgIAAxkBAAEgIWJkRIj-MeWwv364OtXrcsTClGue9AACcg8AAibMiUpsrVGWrFXUvS8E",
        "CAACAgIAAxkBAAEgIWRkRIkbtjVgTqOPklYgo9Vo4Y2_1wACRQsAAsh2GUteeO5PO-ys-y8E",
        "CAACAgIAAxkBAAEgIWhkRIlET04f4SaUmVF2LdU2hZG-EgACzhQAAqOI2Ep-yBn6va_C5C8E",
    ];

    let mut chats = Chats::default();
    let api = API::new(Client::new(env::var("TELEGRAM_TOKEN").unwrap().into()));

    let mut last_update_id = 0;

    loop {
        debug!("last_update_id = {}", last_update_id);
        let updates = api
            .get_events(
                &GetUpdatesRequest::new()
                    .with_timeout(60)
                    .with_offset(last_update_id + 1),
            )
            .await
            .unwrap();

        for update in updates {
            match update {
                Event::NewMessage(id, message) => {
                    last_update_id = max(last_update_id, id);
                    let from = message.from.unwrap();
                    let text = message.text.unwrap();
                    let chat_id = message.chat.id;

                    info!("({}) Message from {}: {}", chat_id, from.first_name, text);

                    let sticker_id = *chats.counters.entry(chat_id).or_insert(0) % stickers.len();

                    api.send_sticker(&SendStickerRequest::new(
                        chat_id,
                        stickers[sticker_id].into(),
                    ))
                    .await
                    .unwrap();

                    chats.counters.insert(chat_id, sticker_id + 1);
                }
                _ => {
                    warn!("Unhandled update: {update:?}");
                }
            }
        }
    }
}
