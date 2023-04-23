#![allow(dead_code)]

#[macro_use]
extern crate log;

use std::{cmp::max, env};

use mogram::{update::GetUpdatesRequest, Client};

#[tokio::main]
async fn main() {
    mogram::init_logger();
    info!("Starting mobot...");

    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap().into());
    client.get_me().await.unwrap();

    let mut last_update_id = 0;

    loop {
        debug!("last_update_id = {}", last_update_id);
        let updates = client
            .get_updates(
                GetUpdatesRequest::new()
                    .with_timeout(60)
                    .with_offset(last_update_id + 1),
            )
            .await
            .unwrap();

        for update in updates {
            last_update_id = max(update.update_id, last_update_id);

            let message = update.message.unwrap();
            let from = message.from.unwrap();
            let text = message.text.unwrap();
            let chat_id = message.chat.id;

            info!("({}) Message from {}: {}", chat_id, from.first_name, text);
        }
    }
}
