use reqwest::Client;
use serde_json::from_str;
use std::{
    env,
    error::Error,
    io::{Write, stdout},
};

use crate::{
    conversation::{AnthropicRequest, ConversationContext, DeltaData, Message},
    spinner::run_with_spinner,
};
use futures_util::StreamExt;

const API_URL: &str = "https://api.openai.com/v1/responses";
const API_CHAT_URL: &str = "https://api.openai.com/v1/chat/completions";
const API_IMG_URL: &str = "https://api.openai.com/v1/images/generations";
const ANTHROPIC_MODELS: &str = "https://api.anthropic.com/v1/models";
const ANTHROPIC_MESSAGES: &str = "https://api.anthropic.com/v1/messages";

pub async fn get_models() -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let response = client
        .get(ANTHROPIC_MODELS)
        .header("x-api-key", env::var("ANTHROPIC_API_KEY").unwrap())
        .header("anthropic-version", "2023-06-01")
        .send()
        .await?
        .text()
        .await?;
    Ok(response)
}

pub async fn stream(context: &mut ConversationContext) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let api_key = env::var("OPENAI_API_KEY").map_err(|_| "OPENAI_API_KEY not set")?;

    println!();
    print!("🤖 ");
    stdout().flush().ok();

    let request_json = serde_json::to_string(context)?;
    let response = client
        .post(API_URL)
        .bearer_auth(&api_key)
        .header("Content-Type", "application/json")
        .body(request_json)
        .send()
        .await?;

    let mut stream = response.bytes_stream();
    let mut acc = String::new();

    while let Some(next) = stream.next().await {
        let next = next?;
        let s = std::str::from_utf8(&next)?;

        // println!("\n\ns:{s}\n\n");

        for p in s.split("data: ") {
            if let Some(real) = p.split("event:").next() {
                if let Ok(d) = serde_json::from_str::<DeltaData>(real.trim()) {
                    print!("{}", d.delta);
                    acc.push_str(&d.delta);
                    stdout().flush().ok();
                }
            }
        }
    }
    context.input.push(Message {
        role: "assistant".into(),
        content: acc,
    });
    println!("\n");
    Ok(())
}

pub async fn anthropic_chat<T>(context: &ConversationContext) -> Result<T, Box<dyn Error>>
where
    T: serde::de::DeserializeOwned,
{
    let client = Client::new();
    let anthropic_request = AnthropicRequest::from_context(context, 2048);
    let request_json = serde_json::to_string(&anthropic_request)?;

    let response_text = run_with_spinner(async {
        client
            .post(ANTHROPIC_MESSAGES)
            .header("Content-Type", "application/json")
            .header("x-api-key", env::var("ANTHROPIC_API_KEY").unwrap())
            .header("anthropic-version", "2023-06-01")
            .body(request_json)
            .send()
            .await?
            .text()
            .await
    })
    .await?;

    print!("\r                \r");
    stdout().flush().ok();

    let resp: T = from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {}\n{}", e, response_text))?;
    Ok(resp)
}

pub async fn send_request<F, T>(url_flag: &str, context: F) -> Result<T, Box<dyn Error>>
where
    F: serde::Serialize,
    T: serde::de::DeserializeOwned,
{
    let client = Client::new();
    let api_key = env::var("OPENAI_API_KEY").map_err(|_| "OPENAI_API_KEY not set")?;

    let mut request_json = serde_json::to_string(&context)?;
    if url_flag == "chat" {
        request_json = request_json.replace("\"input\":", "\"messages\":");
    }
    let url = match url_flag {
        "chat" => API_CHAT_URL,
        "image" => API_IMG_URL,
        _ => API_URL,
    };

    let response_text = run_with_spinner(async {
        client
            .post(url)
            .bearer_auth(&api_key)
            .header("Content-Type", "application/json")
            .body(request_json)
            .send()
            .await?
            .text()
            .await
    })
    .await?;

    print!("\r                \r");
    stdout().flush().ok();

    let resp: T = from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {}\n{}", e, response_text))?;

    Ok(resp)
}
