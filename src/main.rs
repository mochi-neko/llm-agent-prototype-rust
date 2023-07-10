mod api_state;
mod chat;
mod chat_gpt_api;

use std::sync::Arc;

use crate::chat_gpt_api::memory::FiniteQueueMemory;
use api_state::ApiState;
use chat::my_chat::chat_rpc::chat_server::ChatServer;
use chat::my_chat::MyChat;
use chat_gpt_api::specification::{Function, Model};
use tokio::sync::Mutex;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "0.0.0.0:8000".parse()?;

    // create our state
    let model = Model::Gpt35Turbo0613;
    let prompt = "Your are an AI assistant.".to_string();
    let context_memory = FiniteQueueMemory::new(10);
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
    let state = Arc::new(Mutex::new(ApiState {
        model,
        prompt,
        context_memory,
        functions,
    }));

    let chat = MyChat { state };

    let reflection_server = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(chat::my_chat::chat_rpc::FILE_DESCRIPTOR_SET)
        .build()?;

    Server::builder()
        .add_service(ChatServer::new(chat))
        .add_service(reflection_server)
        .serve(address)
        .await?;

    Ok(())
}
