/// This is a simple ping bot that responds to every message with an incrementing counter.
#[macro_use]
extern crate log;

use std::env;

use mobot::{api::SendMessageRequest, *};
use mobot_derive::BotState;

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
/// message containing the counter. We also present the user with an inline
/// keyboard with four buttons.
async fn handle_chat_message(e: Event, state: State<ChatState>) -> Result<Action, anyhow::Error> {
    let mut state = state.get().write().await;

    let chat_id = e.update.chat_id()?;

    state.counter += 1;

    // Remove any reply keyboards that exist...
    e.api
        .send_message(
            &SendMessageRequest::new(chat_id, format!("Pong {}", state.counter))
                .with_reply_markup(api::ReplyMarkup::reply_keyboard_remove()),
        )
        .await?;

    // Send a message with an inline keyboard with four buttons.
    e.api
        .send_message(
            &SendMessageRequest::new(chat_id, "Try again?").with_reply_markup(
                api::ReplyMarkup::inline_keyboard_markup(vec![
                    vec![
                        api::InlineKeyboardButton::from("Again!").with_callback_data("again"),
                        api::InlineKeyboardButton::from("Stop!").with_callback_data("stop"),
                    ],
                    vec![
                        api::InlineKeyboardButton::from("Boo!").with_callback_data("boo"),
                        api::InlineKeyboardButton::from("Blah!").with_callback_data("blah"),
                    ],
                ]),
            ),
        )
        .await?;

    Ok(Action::Done)
}

/// This is called when a button is pressed. We simply acknowledge the button
/// press and send a response to the user.
async fn handle_chat_callback(e: Event, _: State<ChatState>) -> Result<Action, anyhow::Error> {
    // Send a response to the user.
    let response = format!("Okay: {}", e.update.data().unwrap_or("no callback data"));

    e.acknowledge_callback(Some(response)).await?;
    e.remove_inline_keyboard().await?;
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
    Router::new(client)
        // Add a handler to log all events
        .add_route(Route::Default, handlers::log_handler)
        // We add our own handler that responds to messages.
        .add_route(Route::Message(Matcher::Any), handle_chat_message)
        // Add a handler to respond to button presses.
        .add_route(Route::CallbackQuery(Matcher::Any), handle_chat_callback)
        // Start the chat router -- this blocks forever.
        .start()
        .await;
}
