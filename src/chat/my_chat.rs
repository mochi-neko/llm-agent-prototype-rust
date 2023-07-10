pub(crate) mod chat_rpc {
    tonic::include_proto!("chat");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("chat_descriptor");
}

use std::pin::Pin;
use std::sync::Arc;

use crate::api_state::ApiState;
use crate::chat_gpt_api::client::complete_chat;
use crate::chat_gpt_api::memory::Memory;
use crate::chat_gpt_api::specification::{Message, RequestBody, Role};
use async_stream::stream;
use chat_rpc::chat_server::Chat;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

pub struct MyChat {
    pub(crate) state: Arc<Mutex<ApiState>>,
}

#[tonic::async_trait]
impl Chat for MyChat {
    // grpcurl -plaintext -d '{ "message": "Hello!" }' localhost:8000 chat.Chat/CompleteChat
    async fn complete_chat(
        &self,
        request: Request<chat_rpc::ChatRequest>,
    ) -> Result<Response<chat_rpc::ChatResponse>, Status> {
        let mut state = self.state.lock().await;

        let address = request.remote_addr();
        println!(
            "Got a request to complete chat: {:?} from {:?}",
            request, address
        );

        state.context_memory.add(Message {
            role: Role::User.parse_to_string().unwrap(),
            content: Some(request.into_inner().message),
            name: None,
            function_call: None,
        });

        let context = state.context_memory.get();
        let messages = build_messages(state.prompt.clone(), context.clone());

        let parameters: RequestBody = RequestBody {
            model: state.model.parse_to_string().unwrap(),
            messages,
            functions: None,
            function_call: None,
            temperature: None,
            top_p: None,
            n: None,
            stream: None,
            stop: None,
            max_tokens: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
        };

        match complete_chat(parameters, true).await {
            // TODO: Handle errors for each status code
            Err(e) => Err(Status::new(tonic::Code::Unknown, e.to_string())),
            Ok(response) => match response.choices.get(0) {
                None => Err(Status::new(
                    tonic::Code::Internal,
                    "No choices in response".to_string(),
                )),
                Some(choice) => match &choice.message.content {
                    None => Err(Status::new(
                        tonic::Code::Internal,
                        "No content in response".to_string(),
                    )),
                    Some(content) => {
                        state.context_memory.add(Message {
                            role: Role::Assistant.parse_to_string().unwrap(),
                            content: Some(content.to_string()),
                            name: None,
                            function_call: None,
                        });

                        println!(
                            "Responding to complete chat with: {:?} to {:?}",
                            response, address
                        );

                        Ok(Response::new(chat_rpc::ChatResponse {
                            response: content.to_string(),
                        }))
                    }
                },
            },
        }
    }

    type CompleteChatStreamingStream = Pin<
        Box<
            dyn Stream<Item = Result<chat_rpc::ChatStreamingResponse, Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;

    // TODO:
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

fn build_messages(prompt: String, context: Vec<Message>) -> Vec<Message> {
    let mut messages = Vec::new();

    messages.push(Message {
        role: Role::System.parse_to_string().unwrap(),
        content: Some(prompt),
        name: None,
        function_call: None,
    });

    for message in context {
        messages.push(message);
    }

    messages
}
