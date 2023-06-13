use std::collections::hash_map::DefaultHasher;

use mobot_derive::BotRequest;
use serde::{Deserialize, Serialize};

use std::hash::{Hash, Hasher};

use super::API;

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

fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

impl<T: Into<String>> From<T> for User {
    fn from(s: T) -> Self {
        let from = s.into();
        Self {
            id: hash(&from.clone()) as i64,
            first_name: from.clone(),
            last_name: None,
            username: Some(from),
            language_code: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, BotRequest)]
pub struct GetMeRequest {}

impl API {
    pub async fn get_me(&self) -> anyhow::Result<User> {
        let req = GetMeRequest {};
        self.client.post("getMe", &req).await
    }
}
