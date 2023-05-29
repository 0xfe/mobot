/// This is a simple ping bot that responds to every message with an incrementing counter.
#[macro_use]
extern crate log;

use std::env;

use mobot::*;

/// Every Telegram chat session has a unique ID. This is used to identify the
/// chat that the bot is currently in.
///
/// The `ChatState` is a simple counter that is incremented every time a message
/// is received. Every chat session has its own `ChatState`. The `Router` keeps
/// track of the `ChatState` for each chat session.
#[derive(Debug, Clone, Default)]
struct ChatState {
    counter: usize,
}

/// This is our chat handler. We simply increment the counter and reply with a
/// message containing the counter.
async fn handle_chat_event(
    e: chat::Event,
    state: chat::State<ChatState>,
) -> Result<chat::Action, anyhow::Error> {
    let message = e.get_new_message()?.clone();
    let mut state = state.get().write().await;
    state.counter += 1;
    Ok(chat::Action::ReplyText(format!(
        "Pong {}: {}",
        state.counter,
        message.text.unwrap()
    )))
}

#[tokio::main]
async fn main() {
    mobot::init_logger();
    info!("Starting pingbot...");

    // The `Client` is the main entry point to the Telegram API. It is used to
    // send requests to the Telegram API.
    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap().into());

    // The `Router` is the main entry point to the bot. It is used to register
    // handlers for different types of events, and keeps track of the state of
    // the bot, passing it to the right handler.

    // Handler stack:
    // - Respond to messages containing the word "ping" in any case.
    // - Also respond to messages that are exactly "pong" in lowercase
    // - Default route: log the event.
    Router::new(client)
        .add_chat_route(
            Route::NewMessage(Matcher::Regex("[Pp][iI][nN][gG]".into())),
            handle_chat_event,
        )
        .add_chat_route(
            Route::NewMessage(Matcher::Exact("pong".into())),
            handle_chat_event,
        )
        .add_chat_route(Route::Default, chat::log_handler)
        .start()
        .await;
}
