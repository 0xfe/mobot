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

- [`API`] is used to make direct calls to the Telegram API. An instance of `API` is
passed to all handlers within the [`Event`] argument.

- [`Handler`]s are functions that handle events. They are registered with
the [`Router`], and are called when an event is received.

Each [`Handler`] is passed an [`Event`] and a [`State`], and returns an
[`Action`].

- [`Action`]s are the result of [`Handler`] calls. They are used to send
responses to the Telegram API.

- [`Event`]s are the events that the bot receives. They are passed to
[`Handler`]s, and can be used to determine what action to take.

- [`State`] is the user-defined state of the bot. It is passed to `Handler`s, as
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
    let client = Client::new(std::env::var("TELEGRAM_TOKEN").unwrap());
    let mut router = Router::new(client);

    router.add_route(Route::Default, |_, _: State<()>| async move {
        Ok(Action::ReplyText("Hello world!".into()))
    });
    router.start().await;
}
```

# Working with state

Every handler is passed a [`State`] object, which can be used to store
information about the bot. The `State` object is generic, and can be
any type that implements [`Default`], [`Clone`] and [`BotState`] traits.
[`State`]s are typically wrapped in an [`std::sync::Arc`], so that they can be shared between threads.

## Example

In the example below we create a bot that counts the number of messages
sent to it.

```no_run
use mobot::*;

#[derive(Clone, Default, BotState)]
struct App {
   counter: usize,
}

async fn handle_chat_event(e: Event, state: State<App>) -> Result<Action, anyhow::Error> {
  let message = e.update.get_message()?.clone();
  let mut state = state.get().write().await;
  state.counter += 1;
  Ok(Action::ReplyText(format!("Pong {}: {}", state.counter, message.text.unwrap())))
}

#[tokio::main]
async fn main() {
    let client = Client::new(std::env::var("TELEGRAM_TOKEN").unwrap());
    Router::new(client).add_route(Route::Default, handle_chat_event).start().await;
}
```

You can initialize different handlers for different chats, with the `with_state` method:

```no_run
# use mobot::*;
#
# #[derive(Clone, Default, BotState)]
# struct App {}
# impl App {
#    fn new() -> Self {
#        Self {}
#    }
# }
#
# async fn handle_chat_event(e: Event, state: State<App>) -> Result<Action, anyhow::Error> {
#   unreachable!()
# }
# #[tokio::main]
# async fn main() {
#     let client = Client::new(std::env::var("TELEGRAM_TOKEN").unwrap());
#     let mut router = Router::new(client);
#
router
  .with_state(App::new())
  .add_route(Route::Default, handle_chat_event)
  .start().await;
# }
```

# Working with routes

[`Route`]s are used to determine which handler should be called for a given event. Every
`Route` is paired with a [`Matcher`] which is tested against the incoming event. If the
matcher matches, the handler is called. If no matcher matches, the [`Route::Default`] handler
is called. If there are multiple handlers for a route/match pair, then they're executed in
the order they were added.

All routes are passed in the same [`State`] object, so they can share the same state with
each other.

## Example

```no_run
use mobot::*;

async fn handle_ping(e: Event, state: State<()>) -> Result<Action, anyhow::Error> {
    Ok(Action::ReplyText("Pong".into()))
}

async fn handle_any(e: Event, state: State<()>) -> Result<Action, anyhow::Error> {
  match e.update {
    Update::Message(message) => {
      Ok(Action::ReplyText(format!("Got new message: {}", message.text.unwrap())))
    }
    Update::EditedMessage(message) => {
      Ok(Action::ReplyText(format!("Edited message: {}", message.text.unwrap())))
    }
    _ => { unreachable!() }
  }
}

#[tokio::main]
async fn main() {
    let client = Client::new(std::env::var("TELEGRAM_TOKEN").unwrap());
    Router::new(client)
        .add_route(Route::Message(Matcher::Exact("ping".into())), handle_ping)
        .add_route(Route::Message(Matcher::Any), handle_any)
        .add_route(Route::EditedMessage(Matcher::Any), handle_any)
        .add_route(Route::Default, handlers::log_handler)
        .start().await;
}
```

# Working with the Telegram API

## Easy API helpers

Many API calls have helper methods which you can call directly via the handler's [`Event`] parameter, for
example, see how [`Event::send_message`] is used below to send a message to the chat:

```no_run
# use mobot::*;
#
async fn handle_chat_event(e: Event, state: State<()>) -> Result<Action, anyhow::Error> {
    // Get a new direct message or channel post
    let message = e.update.get_message_or_post()?.clone();

    // Reply back to the chat with the same message
    e.send_message(format!("Your message: {}", message.text.unwrap())).await?;
    Ok(Action::Done)
}
```

## Using the Telegram API directly

You can use the [`API`] struct to make calls to the Telegram API. An instance of `API` is
passed to all handlers within the `Event` argument (See [`Event`] and [`Event`]).

## Example

```no_run
# use mobot::*;
#
async fn handle_chat_event(e: Event, state: State<()>) -> Result<Action, anyhow::Error> {
    match e.update {
        Update::Message(message) => {
            e.api
                .send_message(&api::SendMessageRequest::new(
                    message.chat.id, format!("Message: {}", message.text.unwrap())
                )).await?;
        }
        Update::ChannelPost(message) => {
            e.api
                .send_message(&api::SendMessageRequest::new(
                    message.chat.id, format!("Channel post: {}", message.text.unwrap())
                )).await?;
        }
        _ => anyhow::bail!("Unhandled update"),
    }

    Ok(Action::Done)
}
```

MOBOT supports most of the major API calls, however if you need to add more structures or calls, you
can do it by adding a file to `lib/api`. See `lib/api/sticker.rs` for an example of how `sendSticker` was
supported.
 */

#[macro_use]
extern crate log;

pub mod action;
pub mod api;
pub mod client;
pub mod event;
pub mod fake;
pub mod handler;
pub mod handlers;
pub mod progress;
pub mod router;
pub mod text;
pub mod update;

pub use action::Action;
pub use api::api::*;
pub use client::{ApiToken, Client};
pub use event::Event;
pub use handler::{BotHandler, BotHandlerFn, Handler, State};
pub use progress::ProgressBar;
pub use router::{Matcher, Route, Router};
pub use text::Text;
pub use update::Update;

/// Expose mobot_derive macros
pub use mobot_derive::BotRequest;
pub use mobot_derive::BotState;

/// This method initializes [`env_logger`] from the environment, defaulting to `info` level logging.
pub fn init_logger() {
    // We use try_init here so it can by run by tests.
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();
    debug!("Logger initialized.");
}
