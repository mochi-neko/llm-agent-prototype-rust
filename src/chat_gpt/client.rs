use crate::chat_gpt::specification::{Message, Model, RequestBody, Role};
use anyhow::Result;
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use std::env;

pub(crate) async fn complete_chat(
    model: Model,
    prompt: String,
    message: String,
    verbose: bool,
) -> Result<String> {
    let api_key = env::var("OPENAI_API_KEY")?;

    // HTTPS connector
    let https = HttpsConnector::new();

    // Hyper HTTP client with HTTPS support
    let client = Client::builder().build::<_, Body>(https);

    // Create JSON payload
    let reqest_body = RequestBody {
        model: model.parse_to_string()?,
        messages: vec![
            Message {
                role: Role::System.parse_to_string()?,
                content: Some(prompt),
                name: None,
                function_call: None,
            },
            Message {
                role: Role::User.parse_to_string()?,
                content: Some(message),
                name: None,
                function_call: None,
            },
        ],
    };

    // Convert the payload to a string
    let json_str = serde_json::to_string(&reqest_body)?;

    if verbose {
        println!("Request JSON\n{}", json_str);
    }

    // WebAPI URI
    let url = "https://api.openai.com/v1/chat/completions".parse::<hyper::Uri>()?;

    // Create HTTP POST request
    let req = Request::post(url)
        .header("Authorization", "Bearer ".to_owned() + &api_key)
        .header("Content-Type", "application/json")
        .body(Body::from(json_str))?;

    // Make the request
    let response = client.request(req).await?;

    // If the request is successful
    if response.status().is_success() {
        // Read the response body
        let body_bytes = hyper::body::to_bytes(response.into_body()).await?;

        // Convert bytes to string
        let body_string = String::from_utf8(body_bytes.to_vec())?;

        println!("Response JSON:\n{}", body_string);

        // let body_object = serde_json::from_str::<ResponseBody>(&body_string)?;

        Ok(body_string)
    } else {
        eprintln!("HTTP request failed: {}", response.status());

        Err(anyhow::anyhow!("HTTP request failed"))
    }
}
