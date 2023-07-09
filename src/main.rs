use std::pin::Pin;

use async_stream::stream;
use chat::chat_server::{Chat, ChatServer};
use tokio::time::Duration;
use tokio_stream::Stream;
use tonic::{transport::Server, Request, Response, Status};

pub mod chat {
    tonic::include_proto!("chat");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("chat_descriptor");
}

#[derive(Debug, Default)]
pub struct MyChat {}

#[tonic::async_trait]
impl Chat for MyChat {
    // grpcurl -plaintext localhost:8000 chat.Chat/CompleteChat {\n "message": "Hello!" \n}
    async fn complete_chat(
        &self,
        request: Request<chat::ChatRequest>,
    ) -> Result<Response<chat::ChatResponse>, Status> {
        let address = request.remote_addr();
        println!(
            "Got a request to complete chat: {:?} from {:?}",
            request, address
        );

        let response = chat::ChatResponse {
            message: format!("Hello {}!", request.into_inner().message),
        };

        println!(
            "Responding to complete chat with: {:?} to {:?}",
            response, address
        );

        Ok(Response::new(response))
    }

    type CompleteChatStreamingStream = Pin<
        Box<dyn Stream<Item = Result<chat::ChatStreamingResponse, Status>> + Send + Sync + 'static>,
    >;

    async fn complete_chat_streaming(
        &self,
        request: Request<chat::ChatRequest>,
    ) -> Result<Response<Self::CompleteChatStreamingStream>, Status> {
        let address = request.remote_addr();
        println!(
            "Got a request to complete chat streaming: {:?} from {:?}",
            request, address
        );

        let message = request.into_inner().message;

        let output_stream = stream! {
            for i in 0u32..=5 {
                tokio::time::sleep(Duration::from_secs(1)).await;

                let response = chat::ChatStreamingResponse {
                    delta: format!("Hello {}! Count: {}", message, i),
                };

                yield Ok(response);
            }
        };

        println!("Responding to complete chat streaming to {:?}.", address);

        Ok(Response::new(
            Box::pin(output_stream) as Self::CompleteChatStreamingStream
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:8000".parse()?;
    let chat = MyChat::default();

    let reflection_server = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(chat::FILE_DESCRIPTOR_SET)
        .build()?;

    Server::builder()
        .add_service(ChatServer::new(chat))
        .add_service(reflection_server)
        .serve(addr)
        .await?;

    Ok(())
}
