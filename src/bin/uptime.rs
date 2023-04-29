/// This is a simple ping bot that responds to every message with a sticker and
/// a text message.

#[macro_use]
extern crate log;

use std::{env, sync::Arc};

use mogram::{chat, router::*, Client, TelegramClient};
use tokio::{process::Command, sync::Mutex};

/// The state of the chat. This is a simple counter that is incremented every
/// time a message is received.
#[derive(Debug, Clone, Default)]
struct ChatState {
    counter: usize,
}

async fn get_uptime() -> anyhow::Result<String> {
    let child = Command::new("uptime").arg("-p").output();
    let output = child.await?;
    Ok(String::from_utf8(output.stdout)?)
}

/// The handler for the chat. This is a simple function that takes a `ChatEvent`
/// and returns a `ChatAction`.
async fn handle_chat_event<T>(
    e: chat::Event<T>,
    state: Arc<Mutex<ChatState>>,
) -> Result<chat::Action<chat::Op>, chat::Error>
where
    T: TelegramClient,
{
    let mut state = state.lock().await;

    match e.message {
        chat::MessageEvent::New(_) => {
            state.counter += 1;

            Ok(chat::Action::Next(chat::Op::ReplyText(format!(
                "uptime({}): {}",
                state.counter,
                get_uptime()
                    .await
                    .or(Err(chat::Error::Failed("Failed to get uptime".into())))?
            ))))
        }
        _ => Err(chat::Error::Failed("Unhandled update".into())),
    }
}

#[tokio::main]
async fn main() {
    mogram::init_logger();
    info!("Starting pingbot...");

    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap().into());
    let mut router = Router::new(client);

    router.add_chat_handler(chat::log_handler);
    router.add_chat_handler(handle_chat_event);
    router.start().await;
}
