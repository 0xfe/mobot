use anyhow::Result;
use mogram::client::PostFn;
use mogram::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
struct FakeResponse {
    ok: bool,
    method: String,
    request: String,
}

fn post(method: String, req: String) -> Result<String> {
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
    let mut client = Client::new("token".to_string().into());
    client.set_post_fn(PostFn(Box::new(post)));
    let api = API::new(client);

    println!(
        "api = {:#?}",
        api.send_sticker(&SendStickerRequest::new(1, "2".to_string()))
            .await
    );
}
