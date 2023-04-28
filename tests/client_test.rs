use anyhow::Result;
use async_trait::async_trait;
use mogram::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

struct FakeClient;

#[derive(Default, Clone, Serialize, Deserialize)]
struct FakeResponse<R> {
    ok: bool,
    method: String,
    request: R,
}

#[async_trait]
impl TelegramClient for FakeClient {
    async fn post<Req, Resp>(&self, method: &str, req: &Req) -> Result<Resp>
    where
        Req: crate::Request,
        Resp: DeserializeOwned + Clone,
    {
        let request = req.clone();
        let body = serde_json::to_string(&FakeResponse {
            ok: true,
            method: method.to_string(),
            request,
        })
        .unwrap();

        let response = ApiResponse::<Resp>::from_str(&body)?;

        Ok(response.result()?.clone())
    }
}

#[tokio::test]
async fn it_works() {
    let client = FakeClient;
    let api = API::new(client);

    println!(
        "api = {:#?}",
        api.send_sticker(&SendStickerRequest::new(1, "2".to_string()))
            .await
    );
}
