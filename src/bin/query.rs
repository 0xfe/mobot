/// This is a simple ping bot that responds to every message with a sticker and
/// a text message.

#[macro_use]
extern crate log;

use std::{env, sync::Arc};

use mobot::{chat, handlers::query, router::*, Client};
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
async fn handle_query_event(
    _: query::Event,
    state: Arc<Mutex<ChatState>>,
) -> Result<query::Action, anyhow::Error> {
    let mut state = state.lock().await;
    state.counter += 1;

    Ok(query::Action::ReplyText(
        "uptime".into(),
        format!(
            "uptime({}): {}",
            state.counter,
            get_uptime()
                .await
                .or(Err(chat::Error::Failed("Failed to get uptime".into())))?
        ),
    ))
}

#[tokio::main]
async fn main() {
    mobot::init_logger();
    info!("Starting querybot...");

    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap().into());
    let mut router = Router::new(client);

    router.add_query_handler(handle_query_event);
    router.start().await;
}
