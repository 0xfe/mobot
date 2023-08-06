use anyhow::anyhow;
/// This is a simple bot that replies with "Hello world!" to every message.
use mobot::{api::BotCommand, *};
use std::env;

#[derive(Clone, Default, BotState)]
struct App {
    pub message: String,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <message> <token> <token> ...", args[0]);
        return;
    }

    println!("{:?}", args);

    let mut handles = vec![];
    let app = App {
        message: args[1].clone(),
    };

    for token in args.iter().skip(2) {
        let token = token.clone();
        let app = app.clone();

        let handle = tokio::spawn(async move {
            let client = Client::new(token);
            let mut router = Router::new(client).with_state(app);

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

            router.add_route(Route::Default, |e: Event, s: State<App>| async move {
                let message = e.update.get_message_or_post()?;

                if message.text.as_ref().ok_or(anyhow!("bad text"))? == "/help" {
                    return Ok(Action::ReplyText(
                        "This bot does nothing much really :-/".into(),
                    ));
                }

                let message = s.get().read().await.message.clone();
                Ok(Action::ReplyText(message))
            });
            router.start().await;
        });

        handles.push(handle);
    }

    _ = futures::future::join_all(handles).await;
}
