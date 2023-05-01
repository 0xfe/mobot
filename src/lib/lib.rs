//! `mobot` is a Telegram Bot framework for Rust.
//!
//! It supports the full Telegram Bot API, and provides a simple framework
//! around managing routing and state for your bot.
//!
//! # Example
//!
//! ```rust
//! use mogram::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Client::new(std::env::var("TELEGRAM_TOKEN").unwrap().into());
//!     let mut router = Router::new(client);
//!
//!     router.add_chat_handler(|_, _: ()| async move {
//!         Ok(chat::Action::ReplyText("Hello world!".into()))
//!     });
//!     router.start().await;
//! }
//! ```

#[macro_use]
extern crate log;

pub mod api;
pub mod client;
pub mod handlers;
pub mod router;

pub use api::api::*;
pub use client::*;
pub use handlers::*;
pub use router::*;

pub fn init_logger() {
    // We use try_init here so it can by run by tests.
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();
    debug!("Logger initialized.");
}
