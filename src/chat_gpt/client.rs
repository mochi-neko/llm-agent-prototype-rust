use crate::chat_gpt::specification::{RequestBody, ResponseBody};
use anyhow::Result;
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use std::env;

pub(crate) async fn complete_chat(parameters: RequestBody, verbose: bool) -> Result<ResponseBody> {
    let api_key = env::var("OPENAI_API_KEY")?;

    // HTTPS connector
    let https = HttpsConnector::new();

    // Hyper HTTP client with HTTPS support
    let client = Client::builder().build::<_, Body>(https);

    // Serialize the payload to a string
    let json_str = serde_json::to_string(&parameters)?;

    if verbose {
        println!("Request JSON\n{}", json_str);
    }

    // WebAPI URI
    let url = "https://api.openai.com/v1/chat/completions".parse::<hyper::Uri>()?;

    // Create HTTP POST request
    let request = Request::post(url)
        .header("Authorization", "Bearer ".to_owned() + &api_key)
        .header("Content-Type", "application/json")
        .body(Body::from(json_str))?;

    // Make the request
    let response = client.request(request).await?;

    // If the request is successful
    if response.status().is_success() {
        // Read the response body
        let body_bytes = hyper::body::to_bytes(response.into_body()).await?;

        // Convert bytes to string
        let body_string = String::from_utf8(body_bytes.to_vec())?;

        if verbose {
            println!("Response JSON:\n{}", body_string);
        }

        // Deserialize the string to a struct
        let body_object = serde_json::from_str::<ResponseBody>(&body_string)?;

        Ok(body_object)
    } else {
        eprintln!("HTTP request failed: {}", response.status());

        Err(anyhow::anyhow!("HTTP request failed"))
    }
}
