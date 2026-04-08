use reqwest::Client;
use serde_json::from_str;
use std::{env, error::Error, sync::LazyLock, time::Duration};

use crate::{
    conversation::{
        AnthropicMessage, AnthropicRequest, ConversationContext, OpenAIRequest, Provider, ResponseC,
    },
    tc_config::ConfigTC,
};

const API_CHAT_URL: &str = "https://api.openai.com/v1/chat/completions";
const OPENAI_MODELS_URL: &str = "https://api.openai.com/v1/models";
const ANTHROPIC_MODELS: &str = "https://api.anthropic.com/v1/models";
const ANTHROPIC_MESSAGES: &str = "https://api.anthropic.com/v1/messages";

/// Shared HTTP client with connection pooling and configured timeouts
#[allow(clippy::expect_used)]
static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .expect("Failed to create HTTP client")
});

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Send a chat message and return the assistant's reply as a string.
/// Routes to the correct provider based on the model name in `context`.
pub async fn chat(
    context: &ConversationContext,
    config: &ConfigTC,
    max_tokens: Option<usize>,
) -> Result<String, Box<dyn Error>> {
    let provider = Provider::from_model_name(&context.model);

    match provider {
        Provider::Anthropic => chat_anthropic(context, max_tokens).await,
        Provider::OpenAI => {
            let api_key = env::var("OPENAI_API_KEY")
                .map_err(|_| "OPENAI_API_KEY environment variable not set")?;
            chat_openai_compat(
                context,
                API_CHAT_URL,
                &api_key,
                &context.model,
                max_tokens,
            )
            .await
        }
        Provider::Local => {
            let base_url = env::var("TC_LOCAL_URL")
                .ok()
                .or_else(|| config.local_base_url.clone())
                .unwrap_or_else(|| "http://localhost:1234/v1".to_string());
            let url = format!(
                "{}/chat/completions",
                base_url.trim_end_matches('/')
            );
            let model = context
                .model
                .strip_prefix("local/")
                .unwrap_or(&context.model);
            chat_openai_compat(context, &url, "local-no-key-required", model, max_tokens).await
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

async fn chat_anthropic(
    context: &ConversationContext,
    max_tokens: Option<usize>,
) -> Result<String, Box<dyn Error>> {
    let url = env::var("ANTHROPIC_BASE_URL")
        .map(|base| {
            format!(
                "{}/messages",
                base.trim_end_matches('/')
            )
        })
        .unwrap_or_else(|_| ANTHROPIC_MESSAGES.to_string());

    let api_key =
        env::var("ANTHROPIC_API_KEY").map_err(|_| "ANTHROPIC_API_KEY environment variable not set")?;

    let request = AnthropicRequest::from_context(context, max_tokens.unwrap_or(4096));
    let request_json = serde_json::to_string(&request)?;

    let response = HTTP_CLIENT
        .post(&url)
        .header("Content-Type", "application/json")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .body(request_json)
        .send()
        .await?;
    let status = response.status();
    let response_text = response.text().await?;

    if !status.is_success() {
        return Err(format!("Anthropic API returned {}: {}", status, response_text).into());
    }

    let reply: AnthropicMessage = from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {}\n{}", e, response_text))?;

    reply
        .content
        .first()
        .map(|c| c.text.clone())
        .ok_or_else(|| "No content in response".into())
}

async fn chat_openai_compat(
    context: &ConversationContext,
    url: &str,
    api_key: &str,
    model: &str,
    max_tokens: Option<usize>,
) -> Result<String, Box<dyn Error>> {
    let request = OpenAIRequest::new(model, context, max_tokens);
    let request_json = serde_json::to_string(&request)?;

    let http_response = HTTP_CLIENT
        .post(url)
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
        .body(request_json)
        .send()
        .await?;
    let status = http_response.status();
    let response_text = http_response.text().await?;

    if !status.is_success() {
        return Err(format!("API returned {}: {}", status, response_text).into());
    }

    let response: ResponseC = from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {}\n{}", e, response_text))?;

    response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "No content in response".into())
}

// ---------------------------------------------------------------------------
// Model listing (unchanged)
// ---------------------------------------------------------------------------

pub async fn get_anthropic_models() -> Result<String, Box<dyn Error>> {
    let response = HTTP_CLIENT
        .get(ANTHROPIC_MODELS)
        .header("x-api-key", env::var("ANTHROPIC_API_KEY")?)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await?;
    if !status.is_success() {
        return Err(format!("Anthropic models API returned {}: {}", status, body).into());
    }
    Ok(body)
}

pub async fn get_openai_models() -> Result<String, Box<dyn Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let response = HTTP_CLIENT
        .get(OPENAI_MODELS_URL)
        .bearer_auth(&api_key)
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await?;
    if !status.is_success() {
        return Err(format!("OpenAI models API returned {}: {}", status, body).into());
    }
    Ok(body)
}

