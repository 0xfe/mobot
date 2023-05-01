/// This is a simple bot that replies to every message with the current uptime.

#[macro_use]
extern crate log;

use std::{env, sync::Arc};

use mobot::*;
use tokio::{process::Command, sync::Mutex};

/// The state of the chat. This is a simple counter that is incremented every
/// time a message is received.
#[derive(Debug, Clone, Default)]
struct ChatState {
    counter: usize,
}

/// Get the uptime of the system.
async fn get_uptime() -> anyhow::Result<String> {
    let child = Command::new("uptime").arg("-p").output();
    let output = child.await?;
    Ok(String::from_utf8(output.stdout)?)
}

/// The handler for the chat. This is a simple function that takes a `chat::Event`
/// and returns a `chat::Action`. It also receives the current `ChatState` for the
/// chat ID.
async fn handle_chat_event(
    e: chat::Event,
    state: Arc<Mutex<ChatState>>,
) -> Result<chat::Action, anyhow::Error> {
    let mut state = state.lock().await;

    match e.message {
        chat::MessageEvent::New(_) => {
            state.counter += 1;

            Ok(chat::Action::ReplyText(format!(
                "uptime({}): {}",
                state.counter,
                get_uptime()
                    .await
                    .or(Err(chat::Error::Failed("Failed to get uptime".into())))?
            )))
        }
        _ => Err(chat::Error::Failed("Unhandled update".into()).into()),
    }
}

#[tokio::main]
async fn main() {
    mobot::init_logger();
    info!("Starting uptimebot...");

    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap().into());
    let mut router = Router::new(client);

    router.add_chat_handler(chat::log_handler);
    router.add_chat_handler(handle_chat_event);
    router.start().await;
}
