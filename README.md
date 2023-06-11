# MOBOT

[![Crates.io](https://img.shields.io/crates/v/mobot.svg)](https://crates.io/crates/mobot)
[![Documentation](https://docs.rs/mobot/badge.svg)](https://docs.rs/mobot)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`MOBOT` is a Telegram chat framework written in Rust. It comes with a fully native implementation of the
Telegram Bot API.

### Features implemented so far

-   Raw Telegram API with support for Messages, Channels, Stickers, Callbacks, and more.
-   Web framework style Routing API with support for message-based routing and nested handler stacks.
-   Easy application state management. MOBOT makes sure your handler gets the right state for each chat.
-   Integrated test infrastructure (`FakeBot`), to simplify writing unit tests for your bots.
-   Support for progress bars, inline keyboards, "Typing..." indicators, etc. See demo video below.

### Demo Video
This is a demo of a devops bot built with the MOBOT framework.

https://github.com/0xfe/mobot/assets/241299/49ccc77f-1dc5-4319-85c7-0cf0d89e21ff

## Hello World!

Example Bot that replies with "Hello world!" to every message. Working example in `src/bin/hello.rs`.

```rust
use mobot::*;

#[tokio::main]
async fn main() {
    let client = Client::new(std::env::var("TELEGRAM_TOKEN").unwrap().into());
    let mut router = Router::new(client);

    router.add_route(Route::Default, |_, _: State<()>| async move {
        Ok(Action::ReplyText("Hello world!".into()))
    });
    router.start().await;
}
```

## Documentation

See full API documentation at https://docs.rs/mobot.

See examples in [src/bin](https://github.com/0xfe/mobot/tree/main/src/bin).

## Testing

MOBOT is packaged with `fake::FakeAPI`, a library to simplify unit testing your bots. `FakeAPI` can be
plugged into `mobot::Client` using the `with_post_handler` hook. See example below from [router_test.rs](../blob/master/tests/router_test.rs).

```rust
async fn it_works() {
    mobot::init_logger();

    // Create a FakeAPI and attach it to the client. Any Telegram requests are now forwarded
    // to `fakeserver` instead.
    let fakeserver = fake::FakeAPI::new();
    let client = Client::new("token".to_string().into()).with_post_handler(fakeserver.clone());

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
```

## Extending MOBOT

### Adding new Telegram API calls

Adding support for additional APIs is straightforward. It involves creating `struct`s for the request
and response, and adding a method to `API`. For example, to add support for the [sendSticker](https://core.telegram.org/bots/api#sendsticker) Telegram API:

#### Create `SendStickerRequest`

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Sticker {
    /// Unique identifier for this file
    pub file_id: String,

    /// Sticker width
    pub width: i64,

    /// Sticker height
    pub height: i64,

    /// True, if the sticker is animated
    pub is_animated: bool,

    /// Emoji associated with the sticker
    pub emoji: Option<String>,

    /// Name of the sticker set to which the sticker belongs
    pub set_name: Option<String>,

    /// File size
    pub file_size: Option<i64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SendStickerRequest {
    /// Unique identifier for the target chat or username of the target
    pub chat_id: i64,

    /// Sticker to send. Pass a file_id as String to send a file that
    pub sticker: String,

    /// Sends the message silently. Users will receive a notification with
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,

    /// If the message is a reply, ID of the original message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message_id: Option<i64>,
}
```

### Assign the `Request` trait

```rust
impl Request for SendStickerRequest {}
```

### Add the `send_sticker` method call to `API`

```rust

impl API {
    pub async fn send_sticker(&self, req: &SendStickerRequest) -> anyhow::Result<Message> {
        self.client.post("sendSticker", req).await
    }
}
```

### Test and send me a Pull Request

-   Add a test in `tests/`. If necessary update `lib/fake.rs` for client testing.
-   Add example code to `src/bin/`.
-   Commit and send me a PR!

## External Dependencies

This crate requires OpenSSL and `pkg-config`:

-   On Linux: `sudo apt-get install pkg-config libssl-dev`
-   On Mac: nothing to do!

# License

MIT License
Copyright 2023 Mohit Muthanna Cheppudira

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
