#[allow(clippy::module_inception)]
pub mod api;
pub mod chat;
pub mod message;
pub mod query;
pub mod sticker;
pub mod update;
pub mod user;

pub use api::*;
pub use chat::*;
pub use message::*;
pub use query::*;
pub use sticker::*;
pub use update::*;
pub use user::*;
