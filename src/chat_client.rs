use reqwest::Client;
use serde_json::from_str;
use std::{
    env,
    error::Error,
    io::{Write, stdout},
};

use crate::{
    conversation::{ConversationContext, DeltaData, Message},
    spinner::run_with_spinner,
};
use futures_util::StreamExt;

const API_URL: &str = "https://api.openai.com/v1/responses";
const API_CHAT_URL: &str = "https://api.openai.com/v1/chat/completions";
const API_IMG_URL: &str = "https://api.openai.com/v1/images/generations";
// const ANTHROPIC_MODELS: &str = "https://api.anthropic.com/v1/models";
// const ANTHROPIC_MESSAGES: &str = "https://api.anthropic.com/v1/messages";

pub struct ChatClient {
    client: Client,
    api_key: String,
}

impl ChatClient {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let api_key = env::var("OPENAI_API_KEY").map_err(|_| "OPENAI_API_KEY not set")?;
        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    pub async fn stream(&self, context: &mut ConversationContext) -> Result<(), Box<dyn Error>> {
        println!();
        print!("🤖 ");
        stdout().flush().ok();

        let request_json = serde_json::to_string(context)?;
        let response = self
            .client
            .post(API_URL)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
            .body(request_json)
            .send()
            .await?;

        let mut stream = response.bytes_stream();

        let mut acc = String::new();

        while let Some(next) = stream.next().await {
            let next = next?;
            let s = std::str::from_utf8(&next)?;

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

    pub async fn send_request<F, T>(&self, url_flag: &str, context: F) -> Result<T, Box<dyn Error>>
    where
        F: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
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
            self.client
                .post(url)
                .bearer_auth(&self.api_key)
                .header("Content-Type", "application/json")
                .body(request_json)
                .send()
                .await?
                .text()
                .await
        })
        .await?;

        // Clear the spinner from stdout
        print!("\r                \r");
        stdout().flush().ok();

        let resp: T = from_str(&response_text)
            .map_err(|e| format!("Failed to parse response: {}\n{}", e, response_text))?;

        Ok(resp)
    }
}
