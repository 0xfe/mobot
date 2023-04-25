#![allow(dead_code)]

#[macro_use]
extern crate log;

use std::{collections::HashMap, env};

use mogram::{router, Action, ChatAction, ChatEvent, ChatHandler, Client};

#[derive(Debug, Clone, Default)]
struct ChatState {
    counters: HashMap<i64, usize>,
}

#[tokio::main]
async fn main() {
    mogram::init_logger();
    info!("Starting pingbot...");

    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap().into());

    let mut router = router::Router::new(client);

    router.add_chat_handler(ChatHandler::new(|e: ChatEvent, _: ()| async move {
        match e {
            ChatEvent::NewMessage(message) => Ok(Action::Next(ChatAction::ReplyText(format!(
                "pong: {}",
                message.text.unwrap_or_default()
            )))),
            _ => Err(router::Error::Failed("Unhandled update".into())),
        }
    }));

    router.start().await;
}
