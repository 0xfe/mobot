/// This is a simple ping bot that responds to every message with an incrementing counter.
#[macro_use]
extern crate log;

use std::env;

use anyhow::bail;
use mobot::{api::SendMessageRequest, *};

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
    let mut state = state.get().write().await;

    match e.message {
        chat::MessageEvent::New(message) => {
            state.counter += 1;

            // Remove any reply keyboards that exist...
            e.api
                .send_message(
                    &SendMessageRequest::new(message.chat.id, format!("Pong {}", state.counter))
                        .with_reply_markup(api::ReplyMarkup::reply_keyboard_remove()),
                )
                .await?;

            // Send a message with an inline keyboard with four buttons.
            e.api
                .send_message(
                    &SendMessageRequest::new(message.chat.id, "Try again?").with_reply_markup(
                        api::ReplyMarkup::inline_keyboard_markup(vec![
                            vec![
                                api::InlineKeyboardButton::from("Again!")
                                    .with_callback_data("again"),
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

            Ok(chat::Action::Done)
        }

        // This event is triggered when a user clicks on an inline keyboard button.
        chat::MessageEvent::Callback(query) => {
            // Send a response to the user.
            e.api
                .answer_callback_query(&api::AnswerCallbackQueryRequest::new(query.id).with_text(
                    format!(
                        "Okay: {}",
                        query.data.unwrap_or("no callback data".to_string())
                    ),
                ))
                .await?;

            // Clear the inline keyboard.
            if let Some(message) = query.message {
                e.api
                    .edit_message_reply_markup(
                        &api::EditMessageReplyMarkupRequest::new(
                            api::ReplyMarkup::inline_keyboard_markup(vec![vec![]]),
                        )
                        .with_chat_id(message.chat.id)
                        .with_message_id(message.message_id),
                    )
                    .await?;
            }

            Ok(chat::Action::Done)
        }
        _ => bail!("Unhandled update"),
    }
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
    let mut router = Router::new(client);

    // We add a helper handler that logs all incoming messages.
    router.add_chat_handler(chat::log_handler).await;

    // We add our own handler that responds to messages.
    router.add_chat_handler(handle_chat_event).await;

    // Start the chat router -- this blocks forever.
    router.start().await;
}
