use anyhow::Result;
use mobot::{api::API, *};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
struct FakeResponse {
    ok: bool,
    method: String,
    request: String,
}

fn fake_post(method: String, req: String) -> Result<String> {
    let body = serde_json::to_string(&FakeResponse {
        ok: true,
        method,
        request: req,
    })
    .unwrap();

    Ok(body)
}

#[tokio::test]
async fn it_works() {
    let client = Client::new("token".to_string()).with_post_handler_fn(fake_post);
    let api = API::new(client);

    println!(
        "api = {:#?}",
        api.send_sticker(&api::SendStickerRequest::new(1, "2".to_string()))
            .await
    );
}
