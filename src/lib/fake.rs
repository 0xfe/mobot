/// This is a fake Telegram API server. It implements the Telegram API, but
/// instead of sending messages to Telegram, it sends them to a [`FakeChat`] object, which
/// can be used to test bots.
use anyhow::Result;
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{mpsc, Mutex};

use crate::chat::MessageEvent;
use crate::{
    api::{self, GetUpdatesRequest, Message, SendMessageRequest, Update},
    ApiResponse, Post,
};

/// `FakeChat` represents a chat session between a user and a mobot bot. It
/// represents the user side of the chat, and provides methods for sending
/// and receiving events as if a user did.
///
/// You can create a `FakeChat` with [`FakeAPI::create_chat`].
pub struct FakeChat {
    /// The chat ID, you can get this from the router.
    pub chat_id: i64,

    /// The user name of the user (this is converted into a `User` object). The
    /// same name is used as the first name.
    pub from: String,

    /// Internal fields to send receive events from the FakeAPI.
    ///
    /// Events sent here are received by the FakeAPI and sent to the bot (typically)
    /// via the router.
    chat_tx: Arc<tokio::sync::mpsc::Sender<MessageEvent>>,

    /// Events from the bot are received here.
    chat_rx: Arc<Mutex<tokio::sync::mpsc::Receiver<MessageEvent>>>,
}

impl FakeChat {
    /// Send a text message to the bot.
    pub async fn send_text(&self, text: impl Into<String>) -> Result<()> {
        let text = text.into();
        let chat_id = self.chat_id;
        let from = self.from.clone();
        let chat_tx = Arc::clone(&self.chat_tx);

        Ok(chat_tx
            .send(MessageEvent::New(
                FakeMessage::text(chat_id, from, text).into(),
            ))
            .await?)
    }

    /// Edit a text message that was previously sent to the bot.
    pub async fn edit_text(&self, message_id: i64, text: impl Into<String>) -> Result<()> {
        let text = text.into();
        let chat_id = self.chat_id;
        let from = self.from.clone();
        let chat_tx = Arc::clone(&self.chat_tx);

        let mut message: Message = FakeMessage::text(chat_id, from, text).into();
        message.message_id = message_id;

        Ok(chat_tx.send(MessageEvent::Edited(message)).await?)
    }

    /// Wait for an event from the bot. This blocks.
    pub async fn recv_event(&self) -> Option<MessageEvent> {
        let mut rx = self.chat_rx.lock().await;
        rx.recv().await
    }
}

/// `FakeAPI` is a fake Telegram API server. It implements the Telegram API, but instead of
/// sending messages to Telegram, it sends them to a [`FakeChat`] object, which can be used to
/// test bots. `FakeAPI` is used by `Router` via [`FakeServer`].
pub struct FakeAPI {
    /// The username to send responses as.
    pub bot_name: String,

    /// Last update ID.
    pub update_id: Arc<Mutex<i64>>,

    /// Internal fields to send receive events from the FakeChat.
    ///
    ///
    pub chat_tx: Arc<mpsc::Sender<MessageEvent>>,
    pub chat_rx: Arc<Mutex<mpsc::Receiver<MessageEvent>>>,

    /// A map of chat IDs to a channel to send messages to.
    pub chat_map: Arc<Mutex<HashMap<i64, Arc<mpsc::Sender<MessageEvent>>>>>,
}

impl Default for FakeAPI {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeAPI {
    /// Create a new `FakeAPI`.
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        Self {
            bot_name: "mobot".to_string(),
            update_id: Arc::new(Mutex::new(0)),
            chat_tx: Arc::new(tx),
            chat_rx: Arc::new(Mutex::new(rx)),
            chat_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new `FakeChat` object.
    pub async fn create_chat(&self, from: impl Into<String>) -> FakeChat {
        // Create a new Chat ID and channel for this chat session.
        let chat_id = rand::random();
        let (tx, rx) = mpsc::channel(100);

        // Add the session's tx to the chat map.
        self.chat_map.lock().await.insert(chat_id, Arc::new(tx));

        // Create the new fake chat session.
        FakeChat {
            // Generate random chat id
            chat_id,
            from: from.into(),
            chat_tx: Arc::clone(&self.chat_tx),
            chat_rx: Arc::new(Mutex::new(rx)),
        }
    }

    /// Wait for an event from the bot and return it as a standard Telegram update. Typically,
    /// this is called by the router in a loop.
    async fn get_updates(&self, req: GetUpdatesRequest) -> ApiResponse<Vec<Update>> {
        let update_id = {
            let mut update_id = self.update_id.lock().await;
            *update_id += 1;
            *update_id
        };

        // Wait for a message from a chat session.
        let mut rx = self.chat_rx.lock().await;

        tokio::select! {
            Some(msg) = rx.recv() => {
                // Wrap the message in an `Update` and return it back to the caller.
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

    async fn send_message(&self, req: SendMessageRequest) -> ApiResponse<Message> {
        let mut message = Message::fake(self.bot_name.as_str());
        message.chat.id = req.chat_id;
        message.text = Some(req.text);
        message.reply_to_message = req.reply_to_message_id;

        if let Some(chat) = self.chat_map.lock().await.get(&req.chat_id) {
            chat.send(MessageEvent::New(message.clone())).await.unwrap();
        } else {
            warn!("Can't find Chat with id = {}", req.chat_id);
        }

        ApiResponse::Ok(message)
    }

    async fn edit_message_text(&self, req: api::EditMessageTextRequest) -> ApiResponse<Message> {
        let mut message = Message::fake(self.bot_name.as_str());
        message.chat.id = req.base.chat_id.unwrap();
        message.message_id = req.base.message_id.unwrap();
        message.text = Some(req.text);

        if let Some(chat) = self.chat_map.lock().await.get(&message.chat.id) {
            chat.send(MessageEvent::Edited(message.clone()))
                .await
                .unwrap();
        } else {
            warn!("Can't find Chat with id = {}", &message.chat.id);
        }

        ApiResponse::Ok(message)
    }
}

/// `FakeServer` implements the Telegram HTTP API. It forwards reqeusts to [`FakeAPI`].
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
    /// The chat id
    chat_id: i64,

    /// The user who sent the message
    from: String,

    /// The message text
    text: String,
}

impl FakeMessage {
    pub fn text(chat_id: i64, from: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            chat_id,
            from: from.into(),
            text: text.into(),
        }
    }
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
