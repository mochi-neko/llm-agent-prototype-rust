use anyhow::Result;
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use serde_json::json;
use std::env;

pub async fn complete_chat() -> Result<String> {
    let api_key = env::var("OPENAI_API_KEY")?;

    // HTTPS connector
    let https = HttpsConnector::new();

    // Hyper HTTP client with HTTPS support
    let client = Client::builder().build::<_, Body>(https);

    // Create JSON payload
    let json = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant."
            },
            {
                "role": "user",
                "content": "Who won the world series in 2020?"
            }
        ]
    });

    // Convert the payload to a string
    let json_str = json.to_string();

    // WebAPI URL
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

        println!("Response JSON: {}", body_string);

        // let body_object = serde_json::from_str::<ResponseBody>(&body_string)?;

        Ok(body_string)
    } else {
        eprintln!("HTTP request failed: {}", response.status());

        Err(anyhow::anyhow!("HTTP request failed"))
    }
}
