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
#[derive(Clone, Default, BotState)]
struct ChatState {
    counter: usize,
}

/// This is our chat handler. We simply increment the counter and reply with a
/// message containing the counter.
async fn handle_chat_event(e: Event, state: State<ChatState>) -> Result<Action, anyhow::Error> {
    let message = e.update.get_new()?;
    let mut state = state.get().write().await;
    state.counter += 1;
    Ok(Action::ReplyText(format!(
        "Pong {}: {}",
        state.counter,
        message.text.as_ref().unwrap()
    )))
}

#[tokio::main]
async fn main() {
    mobot::init_logger();
    info!("Starting pingbot...");

    // The `Client` is the main entry point to the Telegram API. It is used to
    // send requests to the Telegram API.
    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap());

    // The `Router` is the main entry point to the bot. It is used to register
    // handlers for different types of events, and keeps track of the state of
    // the bot, passing it to the right handler.

    // Handler stack:
    // - Respond to messages containing the word "ping" in any case.
    // - Also respond to messages that are exactly "pong" in lowercase
    // - Default route: log the event.
    Router::new(client)
        .add_route(
            Route::Message(Matcher::Regex("[Pp][iI][nN][gG]".into())),
            handle_chat_event,
        )
        .add_route(
            Route::Message(Matcher::Exact("pong".into())),
            handle_chat_event,
        )
        .add_route(Route::Default, handlers::log_handler)
        .start()
        .await;
}
