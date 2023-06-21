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
    specification::{Function, Message, Model, RequestBody, Role},
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

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct FunctionResponse {
    pub(crate) success: bool,
    pub(crate) info: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) arguments: Option<String>,
}

/// curl http://localhost:8000/chat -X POST -H "Content-Type: application/json" -d '{"message":"Hello!"}'
pub(crate) async fn chat_handler(
    model: extract::Extension<Arc<Model>>,
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
        model: model.parse_to_string().unwrap(),
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

    match complete_chat(parameters, true).await {
        Err(e) => response::Json(ChatResponse {
            message: format!("Error: {:?}", e),
        }),
        Ok(response) => match response.choices.get(0) {
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
        },
    }
}

/// curl http://localhost:8000/chat_stream -X POST -H "Content-Type: application/json" -d '{"message":"Hello!"}'
pub(crate) async fn chat_stream_handler(
    model: extract::Extension<Arc<Model>>,
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
            model: model.parse_to_string().unwrap(),
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

/// curl http://localhost:8000/function -X POST -H "Content-Type: application/json" -d '{"message":"How are you felling now?"}'
pub(crate) async fn function_handler(
    model: extract::Extension<Arc<Model>>,
    memory_state: extract::Extension<Arc<Mutex<FiniteQueueMemory>>>,
    request: extract::Json<ChatRequest>,
) -> response::Json<FunctionResponse> {
    let mut memory = memory_state.lock().await;

    memory.add(Message {
        role: Role::User.parse_to_string().unwrap(),
        content: Some(request.message.to_string()),
        name: None,
        function_call: None,
    });

    let parameters: RequestBody = RequestBody {
        model: model.parse_to_string().unwrap(),
        messages: memory.get(),
        functions: Some(vec![Function {
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
        }]),
        function_call: Some("auto".to_string()),
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
        Err(e) => response::Json(FunctionResponse {
            success: false,
            info: Some(format!("Error: {:?}", e)),
            name: None,
            arguments: None,
        }),
        Ok(response) => match response.choices.get(0) {
            None => response::Json(FunctionResponse {
                success: false,
                info: Some("No choice in response".to_string()),
                name: None,
                arguments: None,
            }),
            Some(choice) => match &choice.message.function_call {
                None => response::Json(FunctionResponse {
                    success: false,
                    info: Some("No function calling in response".to_string()),
                    name: None,
                    arguments: None,
                }),
                Some(function_calling) => {
                    memory.add(Message {
                        role: Role::Assistant.parse_to_string().unwrap(),
                        content: None,
                        name: None,
                        function_call: Some(function_calling.clone()),
                    });
                    response::Json(FunctionResponse {
                        success: true,
                        info: None,
                        name: Some(function_calling.name.clone()),
                        arguments: Some(function_calling.arguments.clone()),
                    })
                }
            },
        },
    }
}
