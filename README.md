# mobot

`mobot` is a Telegram chat framework written in Rust. It comes with a fully native implementation of the
Telegram Bot API.

MIT Licensed. Copyright 2023 Mohit Muthanna Cheppudira.

### Features implemented so far

-   Raw Telegram API with support for Messages, Channels, Stickers, Callbacks, and more.
-   Web framework style Routing API with support for message-based routing and nested handler stacks.
-   Basic test infrastructure (`FakeBot`), to simplify writing unit tests for your bots.

## Hello World!

Bot that replies with "Hello world!" to every message. Working example in `src/bin/hello.rs`.

```rust
use mobot::*;

#[tokio::main]
async fn main() {
    let client = Client::new(std::env::var("TELEGRAM_TOKEN").unwrap().into());
    let mut router = Router::new(client);

    router.add_chat_route(Route::Default, |_, _: chat::State<()>| async move {
        Ok(chat::Action::ReplyText("Hello world!".into()))
    }).await;
    router.start().await;
}
```

## Documentation

See full API documentation at https://docs.rs/mobot.

## External Dependencies

This crate requires OpenSSL and `pkg-config`:

-   On Linux: `sudo apt-get install pkg-config libssl-dev`
-   On Mac: nothing to do!
