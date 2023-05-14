use std::sync::Arc;

use anyhow::{bail, Result};
use async_trait::async_trait;
use log::debug;
use mobot::{api::Update, *};
use tokio::sync::Mutex;

#[derive(Default, Clone)]
struct PostHandler {
    update_id: Arc<Mutex<i64>>,
}

impl PostHandler {
    fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Post for PostHandler {
    async fn post(&self, method: String, req: String) -> Result<String> {
        debug!("method = {}, req = {}", method, req);
        let update_id = {
            let mut lock = self.update_id.lock().await;
            *lock += 1;
            *lock
        };

        let response = match method.as_str() {
            "getUpdates" => serde_json::to_string(&ApiResponse {
                ok: true,
                description: None,
                result: Some(vec![Update {
                    update_id,
                    ..Default::default()
                }]),
            }),
            _ => serde_json::to_string(&ApiResponse::<()> {
                ok: false,
                description: "Unknown method".to_string().into(),
                result: None,
            }),
        };

        let body = response.unwrap();

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
    let client = Client::new("token".to_string().into()).with_post_handler(PostHandler::new());

    let mut router = Router::new(client);

    // We add a helper handler that logs all incoming messages.
    router.add_chat_handler(handle_chat_event).await;

    router.start().await;
}
