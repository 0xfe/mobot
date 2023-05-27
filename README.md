# MOBOT

`MOBOT` is a Telegram chat framework written in Rust. It comes with a fully native implementation of the
Telegram Bot API.

### Features implemented so far

-   Raw Telegram API with support for Messages, Channels, Stickers, Callbacks, and more.
-   Web framework style Routing API with support for message-based routing and nested handler stacks.
-   Basic test infrastructure (`FakeBot`), to simplify writing unit tests for your bots.

## Hello World!

Example Bot that replies with "Hello world!" to every message. Working example in `src/bin/hello.rs`.

```rust
use mobot::*;

#[tokio::main]
async fn main() {
    let client = Client::new(std::env::var("TELEGRAM_TOKEN").unwrap().into());
    let mut router = Router::new(client);

    router.add_chat_route(Route::Default, |_, _: chat::State<()>| async move {
        Ok(chat::Action::ReplyText("Hello world!".into()))
    });
    router.start().await;
}
```

## Documentation

See full API documentation at https://docs.rs/mobot.

See examples in [src/bin](https://github.com/0xfe/mobot/tree/main/src/bin).

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
