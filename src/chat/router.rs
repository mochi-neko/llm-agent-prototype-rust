use axum::{
    extract,
    response::{self, Response},
};
use hyper::Body;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::chat_gpt::{
    client::{complete_chat, complete_chat_stream},
    specification::{Message, Model, RequestBody, Role},
};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ChatRequest {
    pub(crate) message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ChatResponse {
    pub(crate) message: String,
}

pub(crate) async fn chat(payload: extract::Json<ChatResponse>) -> response::Json<ChatResponse> {
    let parameters: RequestBody = RequestBody {
        model: Model::Gpt35Turbo.parse_to_string().unwrap(),
        messages: vec![
            Message {
                role: Role::System.parse_to_string().unwrap(),
                content: Some("あなたは世界的に有名な小説家です。".to_string()),
                name: None,
                function_call: None,
            },
            Message {
                role: Role::User.parse_to_string().unwrap(),
                content: Some(payload.message.to_string()),
                name: None,
                function_call: None,
            },
        ],
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
                    Some(content) => response::Json(ChatResponse {
                        message: content.to_string(),
                    }),
                },
            }
        }
    }
}

pub(crate) async fn chat_stream() -> Response<Body> {
    let (tx, rx) = mpsc::unbounded_channel();

    let parameters = RequestBody {
        model: Model::Gpt35Turbo.parse_to_string().unwrap(),
        messages: vec![
            Message {
                role: Role::System.parse_to_string().unwrap(),
                content: Some("あなたは世界的に有名な小説家です。".to_string()),
                name: None,
                function_call: None,
            },
            Message {
                role: Role::User.parse_to_string().unwrap(),
                content: Some(
                    "「吾輩は猫である」から始まる小説の続きを書いてください。".to_string(),
                ),
                name: None,
                function_call: None,
            },
        ],
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

    tokio::spawn(async move { complete_chat_stream(tx, parameters, true).await });

    Response::builder()
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .header("connection", "keep-alive")
        .body(Body::wrap_stream(UnboundedReceiverStream::new(rx)))
        .unwrap()
}
