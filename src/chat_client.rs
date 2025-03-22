use reqwest::Client;
use serde_json::from_str;
use std::{
    env,
    error::Error,
    io::{Write, stdout},
};

use crate::{
    conversation::{ConversationContext, DeltaData, Message, Response, ResponseC},
    spinner::run_with_spinner,
};
use futures_util::StreamExt;

const API_URL: &str = "https://api.openai.com/v1/responses";
const API_CHAT_URL: &str = "https://api.openai.com/v1/chat/completions";

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
        print!("🤖 ");
        stdout().flush().ok();

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

    pub async fn send_request_c(
        &self,
        context: &ConversationContext,
    ) -> Result<ResponseC, Box<dyn Error>> {
        let request_json = serde_json::to_string(context)?;
        let replaced = request_json.replace("\"input\":", "\"messages\":");
        let response_text = run_with_spinner(async {
            self.client
                .post(API_CHAT_URL)
                .bearer_auth(&self.api_key)
                .header("Content-Type", "application/json")
                .body(replaced)
                .send()
                .await?
                .text()
                .await
        })
        .await?;

        // Clear the spinner from stdout
        print!("\r                \r");
        stdout().flush().ok();
        let resp: ResponseC = from_str(&response_text)
            .map_err(|e| format!("Failed to parse response: {}\n{}", e, response_text))?;

        Ok(resp)
    }

    pub async fn send_request(
        &self,
        context: &ConversationContext,
    ) -> Result<Response, Box<dyn Error>> {
        let request_json = serde_json::to_string(context)?;

        let response_text = run_with_spinner(async {
            self.client
                .post(API_URL)
                .header("Authorization", format!("Bearer {}", self.api_key))
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

        let resp: Response = from_str(&response_text)
            .map_err(|e| format!("Failed to parse response: {}\n{}", e, response_text))?;

        Ok(resp)
    }
}
