/// This example demonstrates how to use the progress bar with a long running operation.
#[macro_use]
extern crate log;

use std::{env, time::Duration};

use mobot::{progress::ProgressBar, *};

// This simulates a long running operation that takes 10 seconds to complete, and
// returns a string.
async fn fun() -> anyhow::Result<String> {
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    Ok(String::from("Done"))
}

/// This is our chat handler. We simply increment the counter and reply with a
/// message containing the counter.
async fn handle_chat_event(e: Event, _: State<()>) -> Result<Action, anyhow::Error> {
    // Run the long running operation while showing a progress bar. Set
    // a 20 second timeout.
    let val = ProgressBar::new()
        .with_timeout(Duration::from_secs(20))
        .start(&e, fun())
        .await?;

    // Send the result back to the user.
    e.send_message(format!("Result: {}", val)).await?;

    Ok(Action::Done)
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
