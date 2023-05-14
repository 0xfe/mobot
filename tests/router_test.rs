use std::{sync::Arc, time::Duration};

use anyhow::{bail, Result};
use async_trait::async_trait;
use log::{debug, info, warn};
use mobot::{
    api::{GetUpdatesRequest, Message, Update},
    *,
};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct TestMessage {
    chat_id: i64,
    text: String,
    from: String,
}

impl From<TestMessage> for Message {
    fn from(m: TestMessage) -> Self {
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

struct FakeChat {
    chat_id: i64,
    from: String,
    chat_tx: Arc<tokio::sync::mpsc::Sender<TestMessage>>,
}

impl FakeChat {
    pub async fn send_message(&self, text: impl Into<String>) -> Result<()> {
        let text = text.into();
        let chat_id = self.chat_id;
        let from = self.from.clone();
        let chat_tx = Arc::clone(&self.chat_tx);

        Ok(chat_tx
            .send(TestMessage {
                chat_id,
                text,
                from,
            })
            .await?)
    }
}

struct PostHandler {
    update_id: Arc<Mutex<i64>>,
    chat_tx: Arc<tokio::sync::mpsc::Sender<TestMessage>>,
    chat_rx: Arc<Mutex<tokio::sync::mpsc::Receiver<TestMessage>>>,
}

impl PostHandler {
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

#[async_trait]
impl Post for PostHandler {
    async fn post(&self, method: String, req: String) -> Result<String> {
        debug!("method = {}, req = {}", method, req);
        let response = match method.as_str() {
            "getUpdates" => self.get_updates(serde_json::from_str(req.as_str())?).await,
            _ => {
                warn!("Unknown method: {}", method);
                ApiResponse::Err(format!("Unknown method: {}", method))
            }
        };

        let body = serde_json::to_string(&response).unwrap();
        Ok(body)
    }
}

#[derive(Debug, Clone, Default)]
struct ChatState {
    counter: i32,
}

/// This is our chat handler. We simply increment the counter and reply with a
/// message containing the counter.
async fn handle_chat_event(
    e: chat::Event,
    state: Arc<Mutex<ChatState>>,
) -> Result<chat::Action, anyhow::Error> {
    let mut state = state.lock().await;

    match e.message {
        chat::MessageEvent::New(message) => {
            state.counter += 1;

            Ok(chat::Action::ReplyText(format!(
                "pong({}): {}",
                state.counter,
                message.text.unwrap_or_default()
            )))
        }
        _ => bail!("Unhandled update"),
    }
}

#[tokio::test]
async fn it_works() {
    mobot::init_logger();
    let fakeserver = PostHandler::new();
    let chat = fakeserver.create_chat("qubyte");
    chat.send_message("ping1").await.unwrap();
    chat.send_message("ping2").await.unwrap();

    let client = Client::new("token".to_string().into()).with_post_handler(fakeserver);

    // Keep the timeout short for testing.
    let mut router = Router::new(client).with_poll_timeout_s(1);

    let (shutdown_notifier, shutdown_tx) = router.shutdown();

    // We add a helper handler that logs all incoming messages.
    router.add_chat_handler(handle_chat_event).await;

    tokio::spawn(async move {
        info!("Starting router...");
        router.start().await;
    });

    tokio::time::sleep(Duration::from_millis(1000)).await;
    info!("Shutting down...");
    shutdown_tx.send(()).await.unwrap();
    shutdown_notifier.notified().await;
}
