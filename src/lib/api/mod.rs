#[allow(clippy::module_inception)]
pub mod api;
pub mod chat;
pub mod format;
pub mod message;
pub mod query;
pub mod reply_markup;
pub mod sticker;
pub mod update;
pub mod user;
pub mod photo_size;
pub mod file;
pub mod document;

pub use api::*;
pub use chat::*;
pub use format::*;
pub use message::*;
pub use query::*;
pub use reply_markup::*;
pub use sticker::*;
pub use update::*;
pub use user::*;
pub use photo_size::*;
pub use file::*;
pub use document::*;