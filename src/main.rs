mod api_state;
mod certification;
mod chat;
mod chat_gpt_api;
mod error_conversion;
mod speak;

use std::sync::Arc;

use crate::api_state::ApiState;
use crate::certification::build_tls_config;
use crate::chat::my_chat::chat_rpc::chat_server::ChatServer;
use crate::chat::my_chat::MyChat;
use crate::chat_gpt_api::memory::FiniteQueueMemory;
use crate::chat_gpt_api::specification::Model;
use crate::speak::my_speak::speak_rpc::speak_server::SpeakServer;
use crate::speak::my_speak::MySpeak;
use tokio::sync::Mutex;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "0.0.0.0:8000".parse()?;

    // create our state
    let model = Model::Gpt35Turbo0613;
    let prompt = "Your are an AI assistant.".to_string();
    let context_memory = FiniteQueueMemory::new(10);
    let state = Arc::new(Mutex::new(ApiState {
        model,
        prompt,
        context_memory,
    }));

    let chat = MyChat {
        state: state.clone(),
    };

    let speak = MySpeak { state };

    let reflection_server = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(chat::my_chat::chat_rpc::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(speak::my_speak::speak_rpc::FILE_DESCRIPTOR_SET)
        .build()?;

    Server::builder()
        .tls_config(build_tls_config()?)?
        .add_service(ChatServer::new(chat))
        .add_service(SpeakServer::new(speak))
        .add_service(reflection_server)
        .serve(address)
        .await?;

    Ok(())
}
