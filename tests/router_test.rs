use std::time::Duration;

use anyhow::{bail, Result};
use log::*;
use mobot::*;

/// `TestApp` represents the state of our test bot. It contains a counter that is incremented
/// every time a message is received.
#[derive(Debug, Clone, Default, BotState)]
struct TestApp {
    counter: i32,
}

/// This is our chat handler. This bot increments the internal counter and replies with a
/// message containing the counter.
async fn handle_chat_event(e: Event, state: State<TestApp>) -> Result<Action, anyhow::Error> {
    let mut state = state.get().write().await;
    match e.update {
        Update::Message(message) => {
            state.counter += 1;

            info!(
                "chatid:{}: pong({}): {}",
                message.chat.id,
                state.counter,
                message.text.clone().unwrap_or_default()
            );
            Ok(Action::ReplyText(format!(
                "pong({}): {}",
                state.counter,
                message.text.unwrap_or_default()
            )))
        }
        Update::EditedMessage(message) => {
            info!(
                "chatid:{}: edited_message: {}",
                message.chat.id,
                message.text.clone().unwrap_or_default()
            );
            Ok(Action::ReplyText(format!(
                "edited_pong: {}",
                message.text.unwrap_or_default()
            )))
        }
        _ => bail!("Unhandled update"),
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
    let mut router = Router::new(client).with_poll_timeout_s(1);

    // Since we're passing ownership of the Router to a background task, grab the
    // shutdown channels so we can shut it down from this task.
    let (shutdown_notifier, shutdown_tx) = router.shutdown();

    // Our bot is a ping bot. Add the handler to the router (see bin/ping.rs).
    router.add_route(Route::Default, handle_chat_event);

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
        "pong(1): ping1"
    );

    // Send the message "ping2", expect the response "pong(2): ping2"
    chat.send_text("ping2").await.unwrap();
    assert_eq!(
        chat.recv_update().await.unwrap().to_string(),
        "pong(2): ping2"
    );

    // Optional: validate there's no more messages from the bot, by waiting two seconds
    // for more messages.
    assert!(
        tokio::time::timeout(Duration::from_millis(2000), chat.recv_update())
            .await
            .is_err()
    );

    // All done shutdown the router, and wait for it to complete.
    info!("Shutting down...");
    shutdown_tx.send(()).await.unwrap();
    shutdown_notifier.notified().await;
}

#[tokio::test]
async fn multiple_chats() {
    mobot::init_logger();
    let fakeserver = fake::FakeAPI::new();
    let client = Client::new("token".to_string()).with_post_handler(fakeserver.clone());

    // Keep the timeout short for testing.
    let mut router = Router::new(client).with_poll_timeout_s(1);
    let (shutdown_notifier, shutdown_tx) = router.shutdown();

    // We add a helper handler that logs all incoming messages.
    router.add_route(Route::Default, handle_chat_event);

    tokio::spawn(async move {
        info!("Starting router...");
        router.start().await;
    });

    let chat1 = fakeserver.create_chat("qubyte").await;
    let chat2 = fakeserver.create_chat("qubyte").await;

    chat1.send_text("ping1").await.unwrap();
    assert_eq!(
        chat1.recv_update().await.unwrap().to_string(),
        "pong(1): ping1"
    );

    chat1.send_text("ping2").await.unwrap();
    assert_eq!(
        chat1.recv_update().await.unwrap().to_string(),
        "pong(2): ping2"
    );

    chat2.send_text("ping1").await.unwrap();
    assert_eq!(
        chat2.recv_update().await.unwrap().to_string(),
        "pong(1): ping1"
    );

    info!("Shutting down...");
    shutdown_tx.send(()).await.unwrap();
    shutdown_notifier.notified().await;
}

#[tokio::test]
async fn multiple_chats_new_state() {
    mobot::init_logger();
    let fakeserver = fake::FakeAPI::new();
    let client = Client::new("token".to_string()).with_post_handler(fakeserver.clone());

    // Keep the timeout short for testing.
    let mut router = Router::new(client)
        .with_poll_timeout_s(1)
        .with_state(TestApp { counter: 1000 });
    let (shutdown_notifier, shutdown_tx) = router.shutdown();

    // We add a helper handler that logs all incoming messages.
    router.add_route(Route::Default, handle_chat_event);

    tokio::spawn(async move {
        info!("Starting router...");
        router.start().await;
    });

    let chat1 = fakeserver.create_chat("qubyte").await;
    let chat2 = fakeserver.create_chat("qubyte").await;

    chat1.send_text("ping1").await.unwrap();
    assert_eq!(
        chat1.recv_update().await.unwrap().to_string(),
        "pong(1001): ping1"
    );

    chat1.send_text("ping2").await.unwrap();
    assert_eq!(
        chat1.recv_update().await.unwrap().to_string(),
        "pong(1002): ping2"
    );

    chat2.send_text("ping1").await.unwrap();
    assert_eq!(
        chat2.recv_update().await.unwrap().to_string(),
        "pong(1001): ping1"
    );

    info!("Shutting down...");
    shutdown_tx.send(()).await.unwrap();
    shutdown_notifier.notified().await;
}

#[tokio::test]
async fn add_route() {
    mobot::init_logger();
    let fakeserver = fake::FakeAPI::new();
    let client = Client::new("token".to_string()).with_post_handler(fakeserver.clone());

    // Keep the timeout short for testing.
    let mut router = Router::new(client).with_poll_timeout_s(1);
    let (shutdown_notifier, shutdown_tx) = router.shutdown();

    // We add a helper handler that logs all incoming messages.
    router
        .add_route(
            Route::Message(Matcher::Prefix("/foo".into())),
            handle_chat_event,
        )
        .add_route(
            Route::Message(Matcher::Exact("boo".into())),
            handle_chat_event,
        );

    tokio::spawn(async move {
        info!("Starting router...");
        router.start().await;
    });

    let chat = fakeserver.create_chat("qubyte").await;

    chat.send_text("ping1").await.unwrap();
    // Wait two seconds for messages -- there should be none, so expect a timeout error.
    assert!(
        tokio::time::timeout(Duration::from_millis(500), chat.recv_update())
            .await
            .is_err()
    );

    chat.send_text("/foobar").await.unwrap();
    assert_eq!(
        chat.recv_update().await.unwrap().to_string(),
        "pong(1): /foobar"
    );

    chat.send_text("boo1").await.unwrap();

    chat.send_text("boo").await.unwrap();
    assert_eq!(
        chat.recv_update().await.unwrap().to_string(),
        "pong(2): boo"
    );

    info!("Shutting down...");
    shutdown_tx.send(()).await.unwrap();
    shutdown_notifier.notified().await;
}

#[tokio::test]
async fn edit_message_text() {
    mobot::init_logger();
    let fakeserver = fake::FakeAPI::new();
    let client = Client::new("token".to_string()).with_post_handler(fakeserver.clone());

    // Keep the timeout short for testing.
    let mut router = Router::new(client).with_poll_timeout_s(1);
    let (shutdown_notifier, shutdown_tx) = router.shutdown();

    // We add a helper handler that logs all incoming messages.
    router.add_route(Route::Default, handle_chat_event);

    tokio::spawn(async move {
        info!("Starting router...");
        router.start().await;
    });

    let chat1 = fakeserver.create_chat("qubyte").await;

    chat1.send_text("ping1").await.unwrap();
    let message: api::Message = chat1.recv_update().await.unwrap().into();

    assert_eq!(message.text.unwrap(), "pong(1): ping1");

    chat1.edit_text(message.message_id, "ping2").await.unwrap();
    assert_eq!(
        chat1.recv_update().await.unwrap().to_string(),
        "edited_pong: ping2"
    );

    info!("Shutting down...");
    shutdown_tx.send(()).await.unwrap();
    shutdown_notifier.notified().await;
}

/// This handler displays a message with two inline keyboard buttons: "yes" and "no".
async fn ask_message(e: Event, _: State<()>) -> Result<Action, anyhow::Error> {
    e.api
        .send_message(
            &api::SendMessageRequest::new(e.update.chat_id()?, "Push the button!")
                .with_parse_mode(api::ParseMode::MarkdownV2)
                .with_reply_markup(api::ReplyMarkup::inline_keyboard_markup(vec![vec![
                    api::InlineKeyboardButton::from("Yes").with_callback_data("yes"),
                    api::InlineKeyboardButton::from("No").with_callback_data("no"),
                ]])),
        )
        .await?;

    Ok(Action::Done)
}

/// This handler is called when one of the buttons above is pressed
async fn ask_callback(e: Event, _: State<()>) -> Result<Action, anyhow::Error> {
    // Handle the callback query from the user. This happens any time a button is pressed
    // on the inline keyboard.
    let action = e.update.data()?;
    e.remove_inline_keyboard().await?;
    Ok(Action::ReplyText(format!("pressed: {}", action)))
}

#[tokio::test]
async fn push_buttons() {
    mobot::init_logger();
    let fakeserver = fake::FakeAPI::new();
    let client = Client::new("token".to_string()).with_post_handler(fakeserver.clone());

    // Keep the timeout short for testing.
    let mut router = Router::new(client).with_poll_timeout_s(1);
    let (shutdown_notifier, shutdown_tx) = router.shutdown();

    // We add a helper handler that logs all incoming messages.
    router.add_route(Route::Message(Matcher::Any), ask_message);
    router.add_route(Route::CallbackQuery(Matcher::Any), ask_callback);

    tokio::spawn(async move {
        info!("Starting router...");
        router.start().await;
    });

    let chat1 = fakeserver.create_chat("qubyte").await;
    chat1.send_text("what?").await.unwrap();

    // Expect some buttons
    let message: api::Message = chat1.recv_update().await.unwrap().into();
    assert_eq!(message.text.unwrap(), "Push the button!");

    // Push "yes"
    chat1.send_callback_query("yes").await.unwrap();
    let event = chat1.recv_update().await.unwrap();

    // Expect the reply markup to be cleared
    let Update::EditedMessage(_) = event else {
        panic!("Expected edited message (reply markup), got {:?}", event);
    };

    // Expect the reply text to be updated with the pressed button: "yes"
    assert_eq!(
        chat1.recv_update().await.unwrap().to_string(),
        "pressed: yes"
    );

    info!("Shutting down...");
    shutdown_tx.send(()).await.unwrap();
    shutdown_notifier.notified().await;
}
