mod chat_gpt;
use crate::chat_gpt::client::complete_chat;
use crate::chat_gpt::specification::Model;
use anyhow::{Ok, Result};
use axum::{routing::get, Router};

#[tokio::main]
async fn main() -> Result<()> {
    // build our application
    let app = Router::new()
        .route("/", get(root))
        .route("/chat", get(chat));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn chat() -> String {
    complete_chat(
        Model::Gpt35Turbo,
        "あなたは世界的に有名な小説家です。".to_string(),
        "「吾輩は猫である」から始まる小説の続きを書いてください。".to_string(),
        true,
    )
    .await
    .unwrap()
}
