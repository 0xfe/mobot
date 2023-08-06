use std::sync::Arc;

use anyhow::Result;
use log::*;
use mobot::{
    api::{SendMessageRequest, API},
    handlers::auth_handler,
    *,
};

/// This is our chat handler. This bot increments the internal counter and replies with a
/// message containing the counter.
async fn handle_chat_event(e: Event, _: State<()>) -> Result<Action, anyhow::Error> {
    Ok(Action::ReplyText(format!(
        "pong: {}",
        e.update.from_user()?.username.as_ref().unwrap()
    )))
}

async fn error_handler<S: handler::BotState>(
    api: Arc<API>,
    chat_id: i64,
    _: State<S>,
    err: anyhow::Error,
) {
    error!("Error: {}", err);
    let result = api
        .send_message(&SendMessageRequest {
            chat_id,
            text: format!("Sorry! {}.", err),
            ..Default::default()
        })
        .await;

    if let Err(err) = result {
        error!("Error in default error handler: {}", err);
    }
}

#[tokio::test]
async fn it_works() {
    mobot::init_logger();

    // Create a FakeAPI and attach it to the client. Any Telegram requests are now forwarded
    // to `fakeserver` instead.
    let fakeserver = fake::FakeAPI::new();
    let client = Client::new("token".to_string()).with_post_handler(fakeserver.clone());

    // Keep the Telegram poll timeout short for testing. The default Telegram poll timeout is 60s.
    let mut router = Router::new(client)
        .with_poll_timeout_s(1)
        .with_error_handler(error_handler);

    // Since we're passing ownership of the Router to a background task, grab the
    // shutdown channels so we can shut it down from this task.
    let (shutdown_notifier, shutdown_tx) = router.shutdown();

    // Our bot is a ping bot. Add the handler to the router (see bin/ping.rs).
    router
        .add_route(Route::Default, auth_handler(vec!["qubyte".to_string()]))
        .add_route(Route::Default, handle_chat_event);

    // Start the router in a background task.
    tokio::spawn(async move {
        info!("Starting router...");
        router.start().await;
    });

    // We're in the foreground. Create a new chat session with the bot, providing your
    // username. This shows up in the `from` field of messages.
    let chat = fakeserver.create_chat("qubyte").await;

    // Send the message "ping1", expect the response "pong(1): ping1"
    chat.send_text("ping1").await.unwrap();
    assert_eq!(
        chat.recv_update().await.unwrap().to_string(),
        "pong: qubyte"
    );

    let chat2 = fakeserver.create_chat("hacker").await;

    // Send the message "ping1", expect the response "pong(1): ping1"
    chat2.send_text("ping1").await.unwrap();
    assert_eq!(
        chat2.recv_update().await.unwrap().to_string(),
        "Sorry! Unauthorized user: hacker."
    );

    // All done shutdown the router, and wait for it to complete.
    info!("Shutting down...");
    shutdown_tx.send(()).await.unwrap();
    shutdown_notifier.notified().await;
}
