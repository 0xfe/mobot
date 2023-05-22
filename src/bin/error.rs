use anyhow::bail;
/// This is a simple bot that replies with "Hello world!" to every message.
use mobot::*;
use std::env;

#[tokio::main]
async fn main() {
    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap().into());

    // Create a router with a custom error handler
    let mut router = Router::new(client).with_error_handler(|api, chat_id, err| async move {
        api.send_message(&api::SendMessageRequest::new(
            chat_id,
            format!("Failed: {}", err),
        ))
        .await
        .unwrap();
    });

    // Return an error from the handler
    router
        .add_chat_handler(
            |_, _: chat::State<()>| async move { bail!("Oh noes! Something went wrong!") },
        )
        .await;

    router.start().await;
}