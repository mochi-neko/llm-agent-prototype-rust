use std::sync::Arc;

use axum::{
    extract,
    response::{self, Response},
};
use hyper::Body;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::chat_gpt::{
    client::{complete_chat, complete_chat_stream},
    specification::{Message, Model, RequestBody, Role},
};

use super::memory::{FiniteQueueMemory, Memory};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ChatRequest {
    pub(crate) message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ChatResponse {
    pub(crate) message: String,
}

/// curl http://localhost:8000/chat -X GET -H "Content-Type: application/json" -d '{"message":"Hello!"}'
pub(crate) async fn chat_handler(
    memory_state: extract::Extension<Arc<Mutex<FiniteQueueMemory>>>,
    request: extract::Json<ChatRequest>,
) -> response::Json<ChatResponse> {
    let mut memory = memory_state.lock().await;

    memory.add(Message {
        role: Role::User.parse_to_string().unwrap(),
        content: Some(request.message.to_string()),
        name: None,
        function_call: None,
    });

    let parameters: RequestBody = RequestBody {
        model: Model::Gpt35Turbo.parse_to_string().unwrap(),
        messages: memory.get(),
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

    let response = complete_chat(parameters, true).await;
    match response {
        Err(e) => response::Json(ChatResponse {
            message: format!("Error: {:?}", e),
        }),
        Ok(response) => {
            let has_choice = response.choices.get(0);
            match has_choice {
                None => response::Json(ChatResponse {
                    message: "No choice in response".to_string(),
                }),
                Some(choice) => match &choice.message.content {
                    None => response::Json(ChatResponse {
                        message: "No content in response".to_string(),
                    }),
                    Some(content) => {
                        memory.add(Message {
                            role: Role::Assistant.parse_to_string().unwrap(),
                            content: Some(content.to_string()),
                            name: None,
                            function_call: None,
                        });
                        response::Json(ChatResponse {
                            message: content.to_string(),
                        })
                    }
                },
            }
        }
    }
}

/// curl http://localhost:8000/chat_stream -X POST -H "Content-Type: application/json" -d '{"message":"Hello!"}'
pub(crate) async fn chat_stream_handler(
    memory_state: extract::Extension<Arc<Mutex<FiniteQueueMemory>>>,
    request: extract::Json<ChatRequest>,
) -> Response<Body> {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let mut memory = memory_state.lock().await;

        memory.add(Message {
            role: Role::User.parse_to_string().unwrap(),
            content: Some(request.message.to_string()),
            name: None,
            function_call: None,
        });

        let parameters = RequestBody {
            model: Model::Gpt35Turbo.parse_to_string().unwrap(),
            messages: memory.get(),
            functions: None,
            function_call: None,
            temperature: None,
            top_p: None,
            n: None,
            stream: Some(true),
            stop: None,
            max_tokens: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
        };

        if let Ok(total_message) = complete_chat_stream(tx, parameters, true).await {
            memory.add(Message {
                role: Role::Assistant.parse_to_string().unwrap(),
                content: Some(total_message),
                name: None,
                function_call: None,
            });
        }
    });

    Response::builder()
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .header("connection", "keep-alive")
        .body(Body::wrap_stream(UnboundedReceiverStream::new(rx)))
        .unwrap()
}
