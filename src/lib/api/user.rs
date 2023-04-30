use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    /// Unique identifier for this user or bot
    pub id: i64,

    /// User‘s or bot’s first name
    pub first_name: String,

    /// User‘s or bot’s last name
    pub last_name: Option<String>,

    /// User‘s or bot’s username
    pub username: Option<String>,

    /// IETF language tag of the user's language
    pub language_code: Option<String>,
}
