use anyhow::Result;
use qdrant_client::qdrant::{vectors::VectorsOptions, ScoredPoint};
use std::sync::Arc;

use axum::{
    extract::{self, State},
    response::{self, Response},
};
use chrono::Utc;
use hyper::{Body, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    api_state::ApiState,
    chat_gpt_api::{
        client::{complete_chat, complete_chat_stream},
        specification::{Message, RequestBody, Role},
    },
    vector_db::{
        database::{DataBase, MetaData},
        tokenizer::detokenize,
    },
};

use super::memory::Memory;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ChatRequest {
    pub(crate) message: String,
    pub(crate) author: String,
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

/// curl http://localhost:8000/chat -X POST -H "Content-Type: application/json" -d '{"message":"Hello!", "author":"mochineko"}'
pub(crate) async fn chat_handler(
    api_state: State<Arc<Mutex<ApiState<'_>>>>,
    request: extract::Json<ChatRequest>,
) -> response::Json<ChatResponse> {
    let mut api_state = api_state.lock().await;

    let session_retrival =
        retrive_vector_memory(&api_state.vector_memories.session, request.message.as_str())
            .await
            .unwrap();

    api_state.context_memory.add(Message {
        role: Role::User.parse_to_string().unwrap(),
        content: Some(request.message.to_string()),
        name: None,
        function_call: None,
    });

    api_state
        .vector_memories
        .session
        .upsert(
            &request.message.to_string(),
            MetaData {
                datetime: Utc::now(),
                author: request.author.clone(),
                addressee: "AI".to_string(), // TODO:
            },
        )
        .await
        .unwrap();

    let context = api_state.context_memory.get();
    let messages = build_messages(api_state.prompt.clone(), session_retrival, context.clone());

    let parameters: RequestBody = RequestBody {
        model: api_state.model.parse_to_string().unwrap(),
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
                    api_state.context_memory.add(Message {
                        role: Role::Assistant.parse_to_string().unwrap(),
                        content: Some(content.to_string()),
                        name: None,
                        function_call: None,
                    });
                    api_state
                        .vector_memories
                        .session
                        .upsert(
                            content,
                            MetaData {
                                datetime: Utc::now(),
                                author: "AI".to_string(), // TODO:
                                addressee: request.author.clone(),
                            },
                        )
                        .await
                        .unwrap();

                    response::Json(ChatResponse {
                        message: content.to_string(),
                    })
                }
            },
        },
    }
}

/// curl http://localhost:8000/chat_stream -X POST -H "Content-Type: application/json" -d '{"message":"Hello!", "author":"mochineko"}'
pub(crate) async fn chat_stream_handler(
    api_state: State<Arc<Mutex<ApiState<'static>>>>,
    request: extract::Json<ChatRequest>,
) -> Response<Body> {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let mut api_state = api_state.lock().await;

        api_state.context_memory.add(Message {
            role: Role::User.parse_to_string().unwrap(),
            content: Some(request.message.to_string()),
            name: None,
            function_call: None,
        });

        api_state
            .vector_memories
            .session
            .upsert(
                &request.message.to_string(),
                MetaData {
                    datetime: Utc::now(),
                    author: request.author.clone(),
                    addressee: "AI".to_string(), // TODO:
                },
            )
            .await
            .unwrap();

        let parameters = RequestBody {
            model: api_state.model.parse_to_string().unwrap(),
            messages: api_state.context_memory.get(),
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
            api_state.context_memory.add(Message {
                role: Role::Assistant.parse_to_string().unwrap(),
                content: Some(total_message.clone()),
                name: None,
                function_call: None,
            });

            api_state
                .vector_memories
                .session
                .upsert(
                    total_message.clone().as_str(),
                    MetaData {
                        datetime: Utc::now(),
                        author: "AI".to_string(), // TODO:
                        addressee: request.author.clone(),
                    },
                )
                .await
                .unwrap();
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
    api_state: State<Arc<Mutex<ApiState<'_>>>>,
    request: extract::Json<ChatRequest>,
) -> response::Json<FunctionResponse> {
    let api_state = api_state.lock().await;
    // Clone not to change original memory
    let mut context_memory = api_state.context_memory.clone();

    context_memory.add(Message {
        role: Role::User.parse_to_string().unwrap(),
        content: Some(request.message.to_string()),
        name: None,
        function_call: None,
    });

    let parameters: RequestBody = RequestBody {
        model: api_state.model.parse_to_string().unwrap(),
        messages: context_memory.get(),
        functions: Some(api_state.functions.to_vec()),
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
                    // Does not record function calling in memory
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

async fn retrive_vector_memory<'a>(database: &DataBase<'a>, query: &str) -> Result<String> {
    let result = database.search(query, 5, None).await?;
    let mut combined = String::new();
    for scored_point in result {
        let vector = extract_vector(&scored_point)?;
        let text = detokenize(vector)?;

        let payload = scored_point.payload;
        let author = payload.get("author").unwrap().to_string();
        let addressee = payload.get("addressee").unwrap().to_string();

        combined.push_str(
            format!(
                "Score = {}, {} -> {}: {}\n",
                scored_point.score, author, addressee, text
            )
            .as_str(),
        );
    }

    Ok(combined)
}

fn extract_vector(point: &ScoredPoint) -> Result<Vec<f32>> {
    if let Some(vectors) = point.vectors.clone() {
        match vectors.vectors_options {
            Some(VectorsOptions::Vector(vector)) => Ok(vector.data),
            _ => Err(anyhow::anyhow!("No vector in result"))?,
        }
    } else {
        Err(anyhow::anyhow!("No vector in result"))?
    }
}

fn build_messages(prompt: String, memory: String, context: Vec<Message>) -> Vec<Message> {
    let mut messages = Vec::new();

    messages.push(Message {
        role: Role::System.parse_to_string().unwrap(),
        content: Some(prompt),
        name: None,
        function_call: None,
    });

    messages.push(Message {
        role: Role::System.parse_to_string().unwrap(),
        content: Some(memory),
        name: None,
        function_call: None,
    });

    for message in context {
        messages.push(message);
    }

    messages
}
