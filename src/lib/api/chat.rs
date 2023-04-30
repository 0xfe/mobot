use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    /// Unique identifier for this chat. This number may be greater than 32 bits and some programming languages may have difficulty/silent defects in interpreting it. But it is smaller than 52 bits, so a signed 64 bit integer or double-precision float type are safe for storing this identifier.
    pub id: i64,

    /// Type of chat, can be either “private”, “group”, “supergroup” or “channel”
    #[serde(rename = "type")]
    pub chat_type: String,

    /// Title, for supergroups, channels and group chats
    pub title: Option<String>,

    /// Username, for private chats, supergroups and channels if available
    pub username: Option<String>,

    /// First name of the other party in a private chat
    pub first_name: Option<String>,

    /// Last name of the other party in a private chat
    pub last_name: Option<String>,

    /// True if a group has ‘All Members Are Admins’ enabled.
    pub all_members_are_administrators: Option<bool>,
}
