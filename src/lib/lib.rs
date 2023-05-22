/*!
`mobot` is a Telegram Bot framework for Rust.

It supports the full Telegram Bot API, and provides a simple framework
around managing routing and state for your bot.

# Framework

The key components of the framework are:

- [`Client`] is the main entry point to the Telegram API. It is used to send
 requests to the Telegram API.

- [`Router`] is the main entry point to the bot. It is used to register
handlers for different types of events, and keeps track of the state of
the bot, passing it to the right handler.

- `Handler`s are functions that handle events. They are registered with
the [`Router`], and are called when an event is received.

Right now there are two types of handlers: [`chat::Handler`] and [`query::Handler`]. The
former is used to handle messages sent to the bot, and the latter is used
to handle inline queries.

Each `Handler` is passed an `Event` and a `State`, and returns an
`Action`.

- `Action`s are the result of `Handler` calls. They are used to send
responses to the Telegram API. See: [`chat::Action`] and [`query::Action`].

- `Event`s are the events that the bot receives. They are passed to
`Handler`s, and can be used to determine what action to take. See [`chat::Event`]
and [`query::Event`].

- `State` is the user-defined state of the bot. It is passed to `Handler`s, as
a generic parameter and can be used to store information about the bot. `State`
must implement the [`Default`] and [`Clone`] traits. [`Default`] is used to
initialize the state of a new chat session, and [`Clone`] is used while passing
the state to the handlers. `State`s are typically wrapped in an [`std::sync::Arc`], so
that they can be shared between threads.

## Example

In the example below we create a bot that replies to every message with the
text "Hello world!".

```no_run
use mobot::*;

#[tokio::main]
async fn main() {
    let client = Client::new(std::env::var("TELEGRAM_TOKEN").unwrap().into());
    let mut router = Router::new(client);

    router.add_chat_handler(|_, _: chat::State<()>| async move {
        Ok(chat::Action::ReplyText("Hello world!".into()))
    });
    router.start().await;
}
```

# Working with state

Every handler is passed a `State` object, which can be used to store
information about the bot. The `State` object is generic, and can be
any type that implements [`Default`] and [`Clone`]. `State`s are typically
wrapped in an [`std::sync::Arc`], so that they can be shared between threads.

## Example

In the example below we create a bot that counts the number of messages
sent to it.

```no_run
#[derive(Clone, Default)]
struct State {
   count: usize,
}

async fn handle_chat_event(
    e: chat::Event,
    state: chat::State<ChatState>,
)    -> Result<chat::Action, anyhow::Error> {
  let mut state = state.get().write().await;
  match e.message {
    chat::MessageEvent::New(message) => {
      state.counter += 1;
      Ok(chat::Action::ReplyText(format!("Pong {}: {}", state.counter, message.text.unwrap())))
    }
    _ => bail!("Unhandled update"),
  }
}

#[tokio::main]
async fn main() {
    let client = Client::new(std::env::var("TELEGRAM_TOKEN").unwrap().into());
    let mut router = Router::new(client);

    router.add_chat_handler(handle_chat_event);
    router.start().await;
}
```

You can initialize different handlers for different chats, with the `with_state` method:

```no_run
router.add_chat_handler(
    chat::Handler::new(handle_chat_event).with_state(App::new(config))
).await;
```

# Working with the Telegram API

You can use the [`API`] struct to make calls to the Telegram API. An instance of `API` is
passed to all handlers within the `Event` argument (See [`chat::Event`] and [`query::Event`]).

## Example

```no_run
async fn handle_chat_event(e: chat::Event, state: chat::State<()>) -> Result<chat::Action, anyhow::Error> {
    let mut state = state.get().write().await;

    match e.message {
        chat::MessageEvent::New(message) => {
            e.api
                .send_message(&SendMessageRequest::new(
                    message.chat.id, format!("Message: {}", message.text.unwrap())
                )).await?;
        }
        chat::MessageEvent::NewPost(message) => {
            e.api
                .send_message(&SendMessageRequest::new(
                    message.chat.id, format!("Channel post: {}", message.text.unwrap())
                )).await?;
        }
        _ => bail!("Unhandled update"),
    }

    Ok(chat::Action::Done)
}
```
 */

#[macro_use]
extern crate log;

pub mod api;
pub mod client;
pub mod fake;
pub mod handlers;
pub mod router;

pub use api::api::*;
pub use client::*;
pub use handlers::*;
pub use router::*;

/// This method initializes [`env_logger`] from the environment, defaulting to `info` level logging.
pub fn init_logger() {
    // We use try_init here so it can by run by tests.
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();
    debug!("Logger initialized.");
}
