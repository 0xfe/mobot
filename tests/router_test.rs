use std::{sync::Arc, time::Duration};

use anyhow::{bail, Result};
use async_trait::async_trait;
use log::{debug, info};
use mobot::{
    api::{GetUpdatesRequest, Message, Update},
    *,
};
use tokio::sync::Mutex;

struct TestMessage {
    chat_id: i64,
    text: String,
    username: String,
}

impl From<TestMessage> for Message {
    fn from(m: TestMessage) -> Self {
        Message {
            from: Some(api::User {
                id: 1,
                first_name: m.username.clone(),
                username: Some(m.username.clone()),
                ..Default::default()
            }),
            chat: api::Chat {
                id: m.chat_id,
                username: Some(m.username),
                ..Default::default()
            },
            text: Some(m.text),
            ..Default::default()
        }
    }
}

#[derive(Default, Clone)]
struct PostHandler {
    update_id: Arc<Mutex<i64>>,
}

impl PostHandler {
    async fn get_updates(&self, req: GetUpdatesRequest) -> ApiResponse<Vec<Update>> {
        let update_id = {
            let mut update_id = self.update_id.lock().await;
            *update_id += 1;
            *update_id
        };

        if update_id > 3 {
            tokio::time::sleep(Duration::from_secs(req.timeout.unwrap_or(1000) as u64)).await;
            return ApiResponse::Ok(vec![]);
        }

        ApiResponse::Ok(vec![Update {
            update_id,
            message: Some(
                TestMessage {
                    chat_id: 1,
                    text: "ping".to_string(),
                    username: "foobar".to_string(),
                }
                .into(),
            ),
            ..Default::default()
        }])
    }
}

#[async_trait]
impl Post for PostHandler {
    async fn post(&self, method: String, req: String) -> Result<String> {
        debug!("method = {}, req = {}", method, req);
        let response = match method.as_str() {
            "getUpdates" => self.get_updates(serde_json::from_str(req.as_str())?).await,
            _ => ApiResponse::Err("Unknown method"),
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
    let client = Client::new("token".to_string().into()).with_post_handler(PostHandler::default());

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
