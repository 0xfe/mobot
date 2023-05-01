# mobot

`mobot` is a telegram chat bot written in Rust. Uses a native implementation of the
Telegram Bot API. Docs at https://docs.rs/mobot.

MIT Licensed. Copyright 2023 Mohit Muthanna Cheppudira.

### Features implemented so far

-   Messages, stickers, and inline queries.
-   Routing API for chat and inline query handlers.
-   Override POST handlers for testability.

## Examples

### Hello World

Bot that replies with "Hello world!" to every message. Working example in `src/bin/hello.rs`.

```rust
use mobot::*;

#[tokio::main]
async fn main() {
    let client = Client::new(std::env::var("TELEGRAM_TOKEN").unwrap().into());
    let mut router = Router::new(client);

    router.add_chat_handler(|_, _: ()| async move {
        Ok(chat::Action::ReplyText("Hello world!".into()))
    });
    router.start().await;
}
```

### Counter Bot - Managing state

This bot demonstrates state managenent in your bots. Working example in `src/bin/ping.rs`.

```rust
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
    state: Arc<Mutex<ChatState>>,
) -> Result<chat::Action, anyhow::Error> {
    let mut state = state.lock().await;

    match e.message {
        chat::MessageEvent::New(message) => {
            state.counter += 1;

            Ok(chat::Action::ReplyText(format!(
                "pong({}): {}",
                state.counter,
                message.text.unwrap_or_default()
            )))
        }
        _ => Err(chat::Error::Failed("Unhandled update".into()).into()),
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
    router.add_chat_handler(chat::log_handler);

    // We add our own handler that responds to messages.
    router.add_chat_handler(handle_chat_event);

    // Start the chat router -- this blocks forever.
    router.start().await;
}
```

### Uptime Bot

Bot that returns server uptime. Working example in `src/bin/uptime.rs`.

```rust
/// The state of the chat. This is a simple counter that is incremented every
/// time a message is received.
#[derive(Debug, Clone, Default)]
struct ChatState {
    counter: usize,
}

/// Get the uptime of the system.
async fn get_uptime() -> anyhow::Result<String> {
    let child = Command::new("uptime").arg("-p").output();
    let output = child.await?;
    Ok(String::from_utf8(output.stdout)?)
}

/// The handler for the chat. This is a simple function that takes a `chat::Event`
/// and returns a `chat::Action`. It also receives the current `ChatState` for the
/// chat ID.
async fn handle_chat_event(e: chat::Event, state: Arc<Mutex<ChatState>>)-> Result<chat::Action, anyhow::Error> {
    let mut state = state.lock().await;

    match e.message {
        // Ignore the chat message, just return the uptime.
        chat::MessageEvent::New(_) => {
            state.counter += 1;

            Ok(chat::Action::ReplyText(format!(
                "uptime({}): {}",
                state.counter,
                get_uptime()
                    .await
                    .or(Err(chat::Error::Failed("Failed to get uptime".into())))?
            )))
        }
        _ => Err(chat::Error::Failed("Unhandled update".into()).into()),
    }
}

#[tokio::main]
async fn main() {
    mobot::init_logger();
    info!("Starting uptimebot...");

    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap().into());
    let mut router = Router::new(client);

    router.add_chat_handler(chat::log_handler);
    router.add_chat_handler(handle_chat_event);
    router.start().await;
}
```

## Testing

Set your telegram API token and then run `bin/hello/rs`.

```
export TELEGRAM_TOKEN=...

RUST_LOG=debug cargo run hello
```

## Dependencies

Need OpenSSL and pkg-config.

```
sudo apt-get install pkg-config libssl-dev
```

## TODO

-   [ ] Process handler return actions
-   [ ] Handler stack
-   [ ] OpenAI integration
