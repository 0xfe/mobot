use serde::{Deserialize, Serialize};

use crate::{Request, API};

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct GetMeRequest {}
impl Request for GetMeRequest {}

impl API {
    pub async fn get_me(&self) -> anyhow::Result<User> {
        let req = GetMeRequest {};
        self.client.post("getMe", &req).await
    }
}
