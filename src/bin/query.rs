/// This is a simple ping bot that responds to every message with a sticker and
/// a text message.

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
struct QueryState {
    counter: usize,
}

async fn get_uptime() -> anyhow::Result<String> {
    let child = Command::new("uptime").arg("-p").output();
    let output = child.await?;
    Ok(String::from_utf8(output.stdout)?)
}

/// The handler for the chat. This is a simple function that takes a `ChatEvent`
/// and returns a `ChatAction`.
async fn handle_query_event(e: Event, state: State<QueryState>) -> Result<Action, anyhow::Error> {
    let mut state = state.get().write().await;
    state.counter += 1;

    e.send_message(format!(
        "uptime({}): {}",
        state.counter,
        get_uptime()
            .await
            .or(Err(anyhow!("Failed to get uptime")))?
    ))
    .await?;

    Ok(Action::Done)
}

#[tokio::main]
async fn main() {
    mobot::init_logger();
    info!("Starting querybot...");

    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap());
    let mut router = Router::new(client);

    router.add_route(Route::InlineQuery(Matcher::Any), handle_query_event);
    router.start().await;
}
