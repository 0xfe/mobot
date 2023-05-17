use std::{sync::Arc, time::Duration};

use anyhow::{bail, Result};
use log::*;
use mobot::{fake::FakeServer, *};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Default)]
struct ChatState {
    counter: i32,
}

/// This is our chat handler. We simply increment the counter and reply with a
/// message containing the counter.
async fn handle_chat_event(
    e: chat::Event,
    state: Arc<RwLock<ChatState>>,
) -> Result<chat::Action, anyhow::Error> {
    let mut state = state.write().await;
    match e.message {
        chat::MessageEvent::New(message) => {
            state.counter += 1;

            info!(
                "chatid:{}: pong({}): {}",
                message.chat.id,
                state.counter,
                message.text.clone().unwrap_or_default()
            );
            Ok(chat::Action::ReplyText(format!(
                "pong({}): {}",
                state.counter,
                message.text.unwrap_or_default()
            )))
        }
        _ => bail!("Unhandled update"),
    }
}

#[tokio::test]
async fn it_works() {
    mobot::init_logger();
    let fakeserver = FakeServer::new();
    let client = Client::new("token".to_string().into()).with_post_handler(fakeserver.clone());

    // Keep the timeout short for testing.
    let mut router = Router::new(client).with_poll_timeout_s(1);
    let (shutdown_notifier, shutdown_tx) = router.shutdown();

    // We add a helper handler that logs all incoming messages.
    router.add_chat_handler(handle_chat_event).await;

    tokio::spawn(async move {
        info!("Starting router...");
        router.start().await;
    });

    let chat = fakeserver.api.create_chat("qubyte").await;

    chat.send_text("ping1").await.unwrap();
    assert_eq!(
        chat.recv_message().await.unwrap().text.unwrap(),
        "pong(1): ping1"
    );

    chat.send_text("ping2").await.unwrap();
    assert_eq!(
        chat.recv_message().await.unwrap().text.unwrap(),
        "pong(2): ping2"
    );

    // Wait two seconds for messages -- there should be none, so expect a timeout error.
    assert!(
        tokio::time::timeout(Duration::from_millis(2000), chat.recv_message())
            .await
            .is_err()
    );

    info!("Shutting down...");
    shutdown_tx.send(()).await.unwrap();
    shutdown_notifier.notified().await;
}

#[tokio::test]
async fn multiple_chats() {
    mobot::init_logger();
    let fakeserver = FakeServer::new();
    let client = Client::new("token".to_string().into()).with_post_handler(fakeserver.clone());

    // Keep the timeout short for testing.
    let mut router = Router::new(client).with_poll_timeout_s(1);
    let (shutdown_notifier, shutdown_tx) = router.shutdown();

    // We add a helper handler that logs all incoming messages.
    router.add_chat_handler(handle_chat_event).await;

    tokio::spawn(async move {
        info!("Starting router...");
        router.start().await;
    });

    let chat1 = fakeserver.api.create_chat("qubyte").await;
    let chat2 = fakeserver.api.create_chat("qubyte").await;

    chat1.send_text("ping1").await.unwrap();
    assert_eq!(
        chat1.recv_message().await.unwrap().text.unwrap(),
        "pong(1): ping1"
    );

    chat1.send_text("ping2").await.unwrap();
    assert_eq!(
        chat1.recv_message().await.unwrap().text.unwrap(),
        "pong(2): ping2"
    );

    chat2.send_text("ping1").await.unwrap();
    assert_eq!(
        chat2.recv_message().await.unwrap().text.unwrap(),
        "pong(1): ping1"
    );

    info!("Shutting down...");
    shutdown_tx.send(()).await.unwrap();
    shutdown_notifier.notified().await;
}
