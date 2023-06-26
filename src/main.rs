mod api_state;
mod chat;
mod chat_gpt_api;
mod vector_db;

use std::collections::VecDeque;
use std::sync::Arc;

use crate::chat::memory::FiniteQueueMemory;
use crate::chat::router::{chat_handler, chat_stream_handler, function_handler};

use crate::api_state::ApiState;
use anyhow::Result;
use axum::{routing::post, Router};
use chat_gpt_api::specification::{Function, Model};
use tokio::sync::Mutex;
use vector_db::vector_memories::VectorMemories;

#[tokio::main]
async fn main() -> Result<()> {
    // create our state
    let model = Model::Gpt35Turbo0613;
    let context_memory = FiniteQueueMemory {
        memories: VecDeque::new(),
        max_size: 10,
    };
    let vector_memories = VectorMemories::new().await?;
    let functions = vec![Function {
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
    }];
    let api_state = Arc::new(Mutex::new(ApiState {
        model,
        context_memory,
        vector_memories,
        functions,
    }));

    // build our application
    let app = Router::new()
        .route("/chat", post(chat_handler))
        .route("/chat_stream", post(chat_stream_handler))
        .route("/function", post(function_handler))
        .with_state(api_state);

    // run it with hyper on localhost:8000
    axum::Server::bind(&"0.0.0.0:8000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
