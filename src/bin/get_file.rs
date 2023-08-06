/// This is a simple bot that download all sent files in directory beside bot executable
use mobot::{
    api::{DownloadRequest, GetFileRequest},
    *,
};
use std::env;

async fn get_user_file(e: Event, _: State<()>) -> Result<Action, anyhow::Error> {
    let telegram_file = e
        .api
        .get_file(&GetFileRequest::new(
            e.update.document().unwrap().file_id.clone(),
        ))
        .await?;
    let mut file = std::fs::File::create(telegram_file.file_id)?;
    let mut content = std::io::Cursor::new(
        e.api
            .download_file(&DownloadRequest::new(telegram_file.file_path.unwrap()))
            .await
            .unwrap(),
    );
    std::io::copy(&mut content, &mut file)?;
    Ok(Action::ReplyText("File saved".into()))
}

#[tokio::main]
async fn main() {
    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap());
    let mut router = Router::<()>::new(client);

    router.add_route(Route::Message(Matcher::Document), get_user_file);
    router.add_route(Route::Message(Matcher::Photo), |_, _| async move {
        Ok(Action::ReplyText("Send a file, not a photo.".into()))
    });
    router.start().await;
}
