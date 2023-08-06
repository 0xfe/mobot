/// This is a simple ping bot that responds to every message with a sticker and
/// a text message.

#[macro_use]
extern crate log;

use std::env;

use lazy_static::lazy_static;
use mobot::*;
use mobot_derive::BotState;

lazy_static! {
    static ref STICKERS: Vec<&'static str> = vec![
        "CAACAgIAAxkBAAEgIVZkRIhFH8RwQ-rH2mAPiT5JyrniMwACqBYAAthnYUhcJMSWynpYVi8E",
        "CAACAgIAAxkBAAEgIU5kRIeMS9WRRu4E8IOpqXf3YrphYQACSgcAAkb7rAQjQpsNX97E4C8E",
        "CAACAgIAAxkBAAEgIVpkRIihHTdCZpqTxIyzNW2Is2LzFQACcAoAAlxVeUsylSm19qsaAAEvBA",
        "CAACAgIAAxkBAAEgIV5kRIjd-5ysEwPs4Npl8RJNVIfjLAACIw4AAp9bSEq43bNL_8rWFi8E",
        "CAACAgIAAxkBAAEgIWJkRIj-MeWwv364OtXrcsTClGue9AACcg8AAibMiUpsrVGWrFXUvS8E",
        "CAACAgIAAxkBAAEgIWRkRIkbtjVgTqOPklYgo9Vo4Y2_1wACRQsAAsh2GUteeO5PO-ys-y8E",
        "CAACAgIAAxkBAAEgIWhkRIlET04f4SaUmVF2LdU2hZG-EgACzhQAAqOI2Ep-yBn6va_C5C8E",
    ];
}

/// The state of the chat. This is a simple counter that is incremented every
/// time a message is received.
#[derive(Clone, Default, BotState)]
struct ChatState {
    counter: usize,
}

/// The handler for the chat. This is a simple function that takes a `ChatEvent`
/// and returns a `ChatAction`.
async fn handle_chat_event(e: Event, state: State<ChatState>) -> Result<Action, anyhow::Error> {
    let mut state = state.get().write().await;
    let message = e.update.get_message()?.clone();
    state.counter += 1;

    e.send_sticker(
        STICKERS
            .get(state.counter % STICKERS.len())
            .unwrap()
            .to_string(),
    )
    .await?;

    Ok(Action::ReplyText(format!(
        "pong({}): {}",
        state.counter,
        message.text.unwrap_or_default()
    )))
}

#[tokio::main]
async fn main() {
    mobot::init_logger();
    info!("Starting pingbot...");

    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap());
    Router::new(client)
        .add_route(Route::Default, handlers::log_handler)
        .add_route(Route::Default, handle_chat_event)
        .start()
        .await;
}
