use anyhow::Result;
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{mpsc, Mutex};

use crate::{
    api::{self, GetUpdatesRequest, Message, SendMessageRequest, Update},
    ApiResponse, Post,
};

pub struct FakeChat {
    pub chat_id: i64,
    pub from: String,
    pub chat_tx: Arc<tokio::sync::mpsc::Sender<FakeMessage>>,
    pub chat_rx: Arc<Mutex<tokio::sync::mpsc::Receiver<Message>>>,
}

impl FakeChat {
    pub async fn send_text(&self, text: impl Into<String>) -> Result<()> {
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

    pub async fn recv_message(&self) -> Option<Message> {
        let mut rx = self.chat_rx.lock().await;
        rx.recv().await
    }
}

pub struct FakeAPI {
    pub update_id: Arc<Mutex<i64>>,
    pub chat_tx: Arc<mpsc::Sender<FakeMessage>>,
    pub chat_rx: Arc<Mutex<mpsc::Receiver<FakeMessage>>>,
    pub chat_queue: Arc<Mutex<HashMap<i64, Arc<mpsc::Sender<Message>>>>>,
}

impl FakeAPI {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        Self {
            update_id: Arc::new(Mutex::new(0)),
            chat_tx: Arc::new(tx),
            chat_rx: Arc::new(Mutex::new(rx)),
            chat_queue: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_chat(&self, from: impl Into<String>) -> FakeChat {
        let chat_id = rand::random();
        let (tx, rx) = mpsc::channel(100);

        self.chat_queue.lock().await.insert(chat_id, Arc::new(tx));

        FakeChat {
            // Generate random chat id
            chat_id,
            from: from.into(),
            chat_tx: Arc::clone(&self.chat_tx),
            chat_rx: Arc::new(Mutex::new(rx)),
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

    async fn send_message(&self, req: SendMessageRequest) -> ApiResponse<Message> {
        let message = Message {
            message_id: rand::random(),
            from: None,
            date: 0,
            chat: api::Chat {
                id: req.chat_id,
                chat_type: String::from("private"),
                username: None,
                first_name: None,
                ..Default::default()
            },
            text: Some(req.text),
            reply_to_message: req.reply_to_message_id,
            ..Default::default()
        };

        if let Some(chat) = self.chat_queue.lock().await.get(&req.chat_id) {
            chat.send(message.clone()).await.unwrap();
        } else {
            warn!("Can't find Chat with id = {}", req.chat_id);
        }

        ApiResponse::Ok(message)
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
        use serde_json::to_string as json;

        debug!("method = {}, req = {}", method, req);
        let response = match method.as_str() {
            "getUpdates" => json(
                &self
                    .api
                    .get_updates(serde_json::from_str(req.as_str())?)
                    .await,
            ),
            "sendMessage" => json(
                &self
                    .api
                    .send_message(serde_json::from_str(req.as_str())?)
                    .await,
            ),
            _ => {
                warn!("Unknown method: {}", method);
                json(&ApiResponse::<()>::Err(format!(
                    "Unknown method: {}",
                    method
                )))
            }
        };

        let body = response.unwrap();
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
