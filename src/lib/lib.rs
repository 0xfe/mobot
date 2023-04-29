#[macro_use]
extern crate log;

pub mod api;
pub mod client;
pub mod event;
pub mod handlers;
pub mod message;
pub mod query;
pub mod router;
pub mod sticker;
pub mod update;

pub use api::*;
pub use client::{Client, TelegramClient};
pub use event::*;
pub use handlers::*;
pub use message::*;
pub use query::*;
pub use router::*;
pub use sticker::*;
pub use update::*;

pub fn init_logger() {
    // We use try_init here so it can by run by tests.
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();
    debug!("Logger initialized.");
}
