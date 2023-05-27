/// This is a fake Telegram API server. It implements the Telegram API, but
/// instead of sending messages to Telegram, it sends them to a [`FakeChat`] object, which
/// can be used to test bots.
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use std::hash::{Hash, Hasher};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    sync::Arc,
    time::Duration,
};
use tokio::sync::{mpsc, Mutex};

use crate::chat::MessageEvent;
use crate::{
    api::{self, GetUpdatesRequest, Message, SendMessageRequest, Update},
    ApiResponse, Post,
};

pub struct FakeChat {
    pub chat_id: i64,
    pub from: String,
    pub chat_tx: Arc<tokio::sync::mpsc::Sender<MessageEvent>>,
    pub chat_rx: Arc<Mutex<tokio::sync::mpsc::Receiver<MessageEvent>>>,
}

impl FakeChat {
    pub async fn send_text(&self, text: impl Into<String>) -> Result<()> {
        let text = text.into();
        let chat_id = self.chat_id;
        let from = self.from.clone();
        let chat_tx = Arc::clone(&self.chat_tx);

        Ok(chat_tx
            .send(MessageEvent::New(
                FakeMessage {
                    chat_id,
                    text,
                    from,
                }
                .into(),
            ))
            .await?)
    }

    pub async fn recv_event(&self) -> Option<MessageEvent> {
        let mut rx = self.chat_rx.lock().await;
        rx.recv().await
    }
}

pub struct FakeAPI {
    pub bot_name: String,
    pub update_id: Arc<Mutex<i64>>,
    pub chat_tx: Arc<mpsc::Sender<MessageEvent>>,
    pub chat_rx: Arc<Mutex<mpsc::Receiver<MessageEvent>>>,
    pub chat_queue: Arc<Mutex<HashMap<i64, Arc<mpsc::Sender<MessageEvent>>>>>,
}

impl Default for FakeAPI {
    fn default() -> Self {
        Self::new()
    }
}

fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

impl FakeAPI {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        Self {
            bot_name: "mobot".to_string(),
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
                match &msg {
                    MessageEvent::New(msg) => {
                        ApiResponse::Ok(vec![Update {
                            update_id,
                            message: Some(msg.clone()),
                            ..Default::default()
                        }])
                    }
                    MessageEvent::Edited(msg) => {
                        ApiResponse::Ok(vec![Update {
                            update_id,
                            edited_message: Some(msg.clone()),
                            ..Default::default()
                        }])
                    }
                    _ => { unimplemented!() }
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(req.timeout.unwrap_or(1000) as u64)) => {
                ApiResponse::Ok(vec![])
            }
        }
    }

    fn create_fake_message(&self) -> Message {
        Message {
            message_id: rand::random(),
            from: Some(api::User {
                id: hash(&self.bot_name.as_str()) as i64,
                first_name: self.bot_name.clone(),
                username: Some(self.bot_name.clone()),
                ..Default::default()
            }),
            date: Utc::now().timestamp(),
            chat: api::Chat {
                chat_type: String::from("private"),
                username: Some(self.bot_name.clone()),
                first_name: Some(self.bot_name.clone()),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    async fn send_message(&self, req: SendMessageRequest) -> ApiResponse<Message> {
        let mut message = self.create_fake_message();
        message.chat.id = req.chat_id;
        message.text = Some(req.text);
        message.reply_to_message = req.reply_to_message_id;

        if let Some(chat) = self.chat_queue.lock().await.get(&req.chat_id) {
            chat.send(MessageEvent::New(message.clone())).await.unwrap();
        } else {
            warn!("Can't find Chat with id = {}", req.chat_id);
        }

        ApiResponse::Ok(message)
    }

    async fn edit_message_text(&self, req: api::EditMessageTextRequest) -> ApiResponse<Message> {
        let mut message = self.create_fake_message();
        message.chat.id = req.base.chat_id.unwrap();
        message.message_id = req.base.message_id.unwrap();
        message.text = Some(req.text);

        if let Some(chat) = self.chat_queue.lock().await.get(&message.chat.id) {
            chat.send(MessageEvent::Edited(message.clone()))
                .await
                .unwrap();
        } else {
            warn!("Can't find Chat with id = {}", &message.chat.id);
        }

        ApiResponse::Ok(message)
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
            "editMessageText" => json(
                &self
                    .api
                    .edit_message_text(serde_json::from_str(req.as_str())?)
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
