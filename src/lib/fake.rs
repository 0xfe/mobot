use anyhow::Result;
use async_trait::async_trait;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

use crate::{
    api::{self, GetUpdatesRequest, Message, Update},
    ApiResponse, Post,
};

pub struct FakeChat {
    pub chat_id: i64,
    pub from: String,
    pub chat_tx: Arc<tokio::sync::mpsc::Sender<FakeMessage>>,
}

impl FakeChat {
    pub async fn send_message(&self, text: impl Into<String>) -> Result<()> {
        let text = text.into();
        let chat_id = self.chat_id;
        let from = self.from.clone();
        let chat_tx = Arc::clone(&self.chat_tx);

        Ok(chat_tx
            .send(FakeMessage {
                chat_id,
                text,
                from,
            })
            .await?)
    }
}

pub struct FakeAPI {
    pub update_id: Arc<Mutex<i64>>,
    pub chat_tx: Arc<tokio::sync::mpsc::Sender<FakeMessage>>,
    pub chat_rx: Arc<Mutex<tokio::sync::mpsc::Receiver<FakeMessage>>>,
}

impl FakeAPI {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        Self {
            update_id: Arc::new(Mutex::new(0)),
            chat_tx: Arc::new(tx),
            chat_rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub fn create_chat(&self, from: impl Into<String>) -> FakeChat {
        FakeChat {
            // Generate random chat id
            chat_id: rand::random(),
            chat_tx: Arc::clone(&self.chat_tx),
            from: from.into(),
        }
    }

    async fn get_updates(&self, req: GetUpdatesRequest) -> ApiResponse<Vec<Update>> {
        let update_id = {
            let mut update_id = self.update_id.lock().await;
            *update_id += 1;
            *update_id
        };

        let mut rx = self.chat_rx.lock().await;

        tokio::select! {
            Some(msg) = rx.recv() => {
                ApiResponse::Ok(vec![Update {
                    update_id,
                    message: Some(msg.into()),
                    ..Default::default()
                }])
            }
            _ = tokio::time::sleep(Duration::from_secs(req.timeout.unwrap_or(1000) as u64)) => {
                ApiResponse::Ok(vec![])
            }
        }
    }
}

impl Default for FakeAPI {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct FakeServer {
    pub api: Arc<FakeAPI>,
}

impl FakeServer {
    pub fn new() -> Self {
        Self {
            api: Arc::new(FakeAPI::new()),
        }
    }
}

impl Default for FakeServer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Post for FakeServer {
    async fn post(&self, method: String, req: String) -> Result<String> {
        debug!("method = {}, req = {}", method, req);
        let response = match method.as_str() {
            "getUpdates" => {
                self.api
                    .get_updates(serde_json::from_str(req.as_str())?)
                    .await
            }
            _ => {
                warn!("Unknown method: {}", method);
                ApiResponse::Err(format!("Unknown method: {}", method))
            }
        };

        let body = serde_json::to_string(&response).unwrap();
        Ok(body)
    }
}

#[derive(Debug, Clone)]
pub struct FakeMessage {
    chat_id: i64,
    text: String,
    from: String,
}

impl From<FakeMessage> for api::Message {
    fn from(m: FakeMessage) -> Self {
        Message {
            from: Some(api::User {
                id: 1,
                first_name: m.from.clone(),
                username: Some(m.from.clone()),
                ..Default::default()
            }),
            chat: api::Chat {
                id: m.chat_id,
                username: Some(m.from),
                ..Default::default()
            },
            text: Some(m.text),
            ..Default::default()
        }
    }
}
