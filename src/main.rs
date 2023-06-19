mod chat;
mod chat_gpt;
use crate::chat::router::{chat, chat_stream};
use anyhow::Result;
use axum::{routing::get, Router};

#[tokio::main]
async fn main() -> Result<()> {
    // build our application
    let app = Router::new()
        .route("/", get(root))
        .route("/chat", get(chat))
        .route("/chat_stream", get(chat_stream));

    // run it with hyper on localhost:8000
    axum::Server::bind(&"0.0.0.0:8000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}
