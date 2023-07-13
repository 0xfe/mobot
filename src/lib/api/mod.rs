#[allow(clippy::module_inception)]
pub mod api;
pub mod botcommand;
pub mod chat;
pub mod document;
pub mod file;
pub mod format;
pub mod message;
pub mod photo_size;
pub mod query;
pub mod reply_markup;
pub mod sticker;
pub mod update;
pub mod user;

pub use api::*;
pub use botcommand::*;
pub use chat::*;
pub use document::*;
pub use file::*;
pub use format::*;
pub use message::*;
pub use photo_size::*;
pub use query::*;
pub use reply_markup::*;
pub use sticker::*;
pub use update::*;
pub use user::*;
