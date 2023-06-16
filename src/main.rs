mod chat_gpt;
use crate::chat_gpt::client::complete_chat;
use crate::chat_gpt::specification::Model;
use anyhow::Result;
use axum::{response, routing::get, Router};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<()> {
    // build our application
    let app = Router::new()
        .route("/", get(root))
        .route("/chat", get(chat));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatResponse {
    message: String,
}

async fn chat() -> response::Json<ChatResponse> {
    let response = complete_chat(
        Model::Gpt35Turbo,
        "あなたは世界的に有名な小説家です。".to_string(),
        "「吾輩は猫である」から始まる小説の続きを書いてください。".to_string(),
        true,
    )
    .await;
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
