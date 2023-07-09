pub(crate) mod chat_rpc {
    tonic::include_proto!("chat");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("chat_descriptor");
}

use std::pin::Pin;

use async_stream::stream;
use chat_rpc::chat_server::Chat;
use tokio::time::Duration;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct MyChat {}

#[tonic::async_trait]
impl Chat for MyChat {
    // grpcurl -plaintext localhost:8000 chat.Chat/CompleteChat {\n "message": "Hello!" \n}
    async fn complete_chat(
        &self,
        request: Request<chat_rpc::ChatRequest>,
    ) -> Result<Response<chat_rpc::ChatResponse>, Status> {
        let address = request.remote_addr();
        println!(
            "Got a request to complete chat: {:?} from {:?}",
            request, address
        );

        let response = chat_rpc::ChatResponse {
            message: format!("Hello {}!", request.into_inner().message),
        };

        println!(
            "Responding to complete chat with: {:?} to {:?}",
            response, address
        );

        Ok(Response::new(response))
    }

    type CompleteChatStreamingStream = Pin<
        Box<
            dyn Stream<Item = Result<chat_rpc::ChatStreamingResponse, Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;

    // grpcurl -plaintext localhost:8000 chat.Chat/CompleteChatStreaming {\n "message": "Hello!" \n}
    async fn complete_chat_streaming(
        &self,
        request: Request<chat_rpc::ChatRequest>,
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

                let response = chat_rpc::ChatStreamingResponse {
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
