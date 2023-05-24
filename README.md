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

    router.add_chat_handler(|_, _: chat::State<()>| async move {
        Ok(chat::Action::ReplyText("Hello world!".into()))
    }).await;
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
    state: chat::State<ChatState>,
) -> Result<chat::Action, anyhow::Error> {
    let mut state = state.get().write().await;

    match e.message {
        chat::MessageEvent::New(message) => {
            state.counter += 1;

            Ok(chat::Action::ReplyText(format!(
                "pong({}): {}",
                state.counter,
                message.text.unwrap_or_default()
            )))
        }
        _ => anyhow::bail!("Unhandled update"),
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

-   [ ] Improve routing
    -   [ ] Routing by query callback data
    -   [ ] Routing by message type: new, edited, etc.
    -   [ ] Routing by state
    -   [ ] Routing by bot command
    -   [ ] Routing by message content
-   [x] Process handler return actions
-   [x] Handler stack
-   [x] OpenAI integration
