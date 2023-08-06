/// This is a simple bot that replies to every message with the current uptime.

#[macro_use]
extern crate log;

use std::env;

use anyhow::anyhow;
use mobot::*;
use mobot_derive::BotState;
use tokio::process::Command;

/// The state of the chat. This is a simple counter that is incremented every
/// time a message is received.
#[derive(Clone, Default, BotState)]
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
async fn handle_chat_event(e: Event, state: State<ChatState>) -> Result<Action, anyhow::Error> {
    let mut state = state.get().write().await;
    state.counter += 1;

    // Show a "Typing..." action while we process the request.
    e.send_chat_action(api::ChatAction::Typing).await?;

    Ok(Action::ReplyText(format!(
        "uptime({}): {}",
        state.counter,
        get_uptime()
            .await
            .or(Err(anyhow!("Failed to get uptime")))?
    )))
}

#[tokio::main]
async fn main() {
    mobot::init_logger();
    info!("Starting uptimebot...");

    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap());
    Router::new(client)
        .add_route(Route::Default, handlers::log_handler)
        .add_route(Route::Default, handle_chat_event)
        .start()
        .await;
}
