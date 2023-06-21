mod chat;
mod chat_gpt;
use std::collections::VecDeque;
use std::sync::Arc;

use crate::chat::memory::FiniteQueueMemory;
use crate::chat::router::{chat_handler, chat_stream_handler, function_handler};
use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use chat_gpt::specification::{Function, Model};
use tokio::sync::Mutex;
use tower_http::add_extension::AddExtensionLayer;

#[tokio::main]
async fn main() -> Result<()> {
    // create our state
    let model = Arc::new(Model::Gpt35Turbo0613);
    let memory_state = Arc::new(Mutex::new(FiniteQueueMemory {
        memories: VecDeque::new(),
        max_size: 10,
    }));
    let functions = Arc::new(vec![Function {
        name: "emotion_simulator".to_string(),
        description: Some("Simulate emotion of AI like human.".to_string()),
        parameters: Some(
            serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(
                r#"{
                    "type": "object",
                    "properties": {
                        "emotion": {
                            "type": "string",
                            "enum": [
                                "neutral",
                                "happy",
                                "sad",
                                "angry",
                                "surprised",
                                "disgusted",
                                "fearful"
                            ]
                        }
                    },
                    "required": [
                        "emotion"
                    ]
                }"#,
            )
            .unwrap(),
        ),
    }]);

    // build our application
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/chat", post(chat_handler))
        .route("/chat_stream", post(chat_stream_handler))
        .route("/function", post(function_handler))
        .layer(AddExtensionLayer::new(model))
        .layer(AddExtensionLayer::new(memory_state))
        .layer(AddExtensionLayer::new(functions));

    // run it with hyper on localhost:8000
    axum::Server::bind(&"0.0.0.0:8000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// curl http://localhost:8000/
async fn root_handler() -> &'static str {
    "Hello, World!"
}
