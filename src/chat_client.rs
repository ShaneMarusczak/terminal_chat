use reqwest::Client;
use serde_json::from_str;
use std::{
    env,
    error::Error,
    io::{Write, stdout},
    sync::LazyLock,
    time::Duration,
};

use crate::{
    conversation::{AnthropicRequest, ConversationContext, OpenAIRequest, ResponsesRequest},
    spinner::run_with_spinner,
};

const API_URL: &str = "https://api.openai.com/v1/responses";
const API_CHAT_URL: &str = "https://api.openai.com/v1/chat/completions";
const API_IMG_URL: &str = "https://api.openai.com/v1/images/generations";
const OPENAI_MODELS_URL: &str = "https://api.openai.com/v1/models";
const ANTHROPIC_MODELS: &str = "https://api.anthropic.com/v1/models";
const ANTHROPIC_MESSAGES: &str = "https://api.anthropic.com/v1/messages";

/// Shared HTTP client with connection pooling and configured timeouts
#[allow(clippy::expect_used)]
static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(300)) // 5 minute timeout for long AI responses
        .build()
        .expect("Failed to create HTTP client")
});

pub async fn get_anthropic_models() -> Result<String, Box<dyn Error>> {
    let response = HTTP_CLIENT
        .get(ANTHROPIC_MODELS)
        .header("x-api-key", env::var("ANTHROPIC_API_KEY")?)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await?
        .text()
        .await?;
    Ok(response)
}

pub async fn get_openai_models() -> Result<String, Box<dyn Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let response = HTTP_CLIENT
        .get(OPENAI_MODELS_URL)
        .bearer_auth(&api_key)
        .send()
        .await?
        .text()
        .await?;
    Ok(response)
}

pub async fn anthropic_chat<T>(context: &ConversationContext) -> Result<T, Box<dyn Error>>
where
    T: serde::de::DeserializeOwned,
{
    let anthropic_request = AnthropicRequest::from_context(context, 2048);
    let request_json = serde_json::to_string(&anthropic_request)?;
    let api_key = env::var("ANTHROPIC_API_KEY")?;

    let response_text = run_with_spinner(async {
        HTTP_CLIENT
            .post(ANTHROPIC_MESSAGES)
            .header("Content-Type", "application/json")
            .header("x-api-key", api_key)
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

pub async fn send_request<T>(
    url_flag: &str,
    context: &ConversationContext,
) -> Result<T, Box<dyn Error>>
where
    T: serde::de::DeserializeOwned,
{
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY environment variable not set")?;

    let (request_json, url) = match url_flag {
        "chat" => {
            let openai_request = OpenAIRequest::from_context(context);
            (serde_json::to_string(&openai_request)?, API_CHAT_URL)
        }
        _ => {
            let responses_request = ResponsesRequest::from_context(context);
            (serde_json::to_string(&responses_request)?, API_URL)
        }
    };

    let response_text = run_with_spinner(async {
        HTTP_CLIENT
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

pub async fn send_image_request<F, T>(request: F) -> Result<T, Box<dyn Error>>
where
    F: serde::Serialize,
    T: serde::de::DeserializeOwned,
{
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY environment variable not set")?;

    let request_json = serde_json::to_string(&request)?;

    let response_text = run_with_spinner(async {
        HTTP_CLIENT
            .post(API_IMG_URL)
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
