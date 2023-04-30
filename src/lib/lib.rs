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
