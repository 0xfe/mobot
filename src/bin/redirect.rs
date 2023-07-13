use anyhow::anyhow;
/// This is a simple bot that replies with "Hello world!" to every message.
use mobot::{api::BotCommand, *};
use std::env;

#[tokio::main]
async fn main() {
    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap().into());
    let mut router = Router::<()>::new(client);

    let commands = vec![
        BotCommand {
            command: "start".into(),
            description: "Start the bot".into(),
        },
        BotCommand {
            command: "help".into(),
            description: "Show help".into(),
        },
    ];

    router
        .api
        .set_my_commands(&api::SetMyCommandsRequest {
            commands,
            ..Default::default()
        })
        .await
        .unwrap();

    router.add_route(Route::Default, |e: Event, _| async move {
        let message = e.update.get_message_or_post()?;

        if message.text.as_ref().ok_or(anyhow!("bad text"))? == "/help" {
            return Ok(Action::ReplyText(
                "This bot does nothing much really :-/".into(),
            ));
        }

        Ok(Action::ReplyText(
            "This bot is now at @rudewordlebot".into(),
        ))
    });
    router.start().await;
}
