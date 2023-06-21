mod chat;
mod chat_gpt;
use std::collections::VecDeque;
use std::sync::Arc;

use crate::chat::memory::FiniteQueueMemory;
use crate::chat::router::{chat_handler, chat_stream_handler};
use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use chat_gpt::specification::Model;
use tokio::sync::Mutex;
use tower_http::add_extension::AddExtensionLayer;

#[tokio::main]
async fn main() -> Result<()> {
    // create our state
    let model = Arc::new(Model::Gpt35Turbo);
    let memory_state = Arc::new(Mutex::new(FiniteQueueMemory {
        memories: VecDeque::new(),
        max_size: 10,
    }));

    // build our application
    let app = Router::new()
        .route("/", get(root))
        .route("/chat", post(chat_handler))
        .route("/chat_stream", post(chat_stream_handler))
        .layer(AddExtensionLayer::new(model))
        .layer(AddExtensionLayer::new(memory_state));

    // run it with hyper on localhost:8000
    axum::Server::bind(&"0.0.0.0:8000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// curl http://localhost:8000/
async fn root() -> &'static str {
    "Hello, World!"
}
