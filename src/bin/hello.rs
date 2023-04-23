#![allow(dead_code)]
use std::env;

use mogram::Client;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap().into());
    client.get_me().await.unwrap();
    println!("{:?}", client.get_updates().await.unwrap());
}
