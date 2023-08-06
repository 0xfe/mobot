/// This is a simple bot that replies with "Hello world!" to every message.
use mobot::*;
use std::env;

#[tokio::main]
async fn main() {
    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap());
    let mut router = Router::<()>::new(client);

    router.add_route(Route::Default, |_, _| async move {
        Ok(Action::ReplyText("Hello world!".into()))
    });
    router.start().await;
}
