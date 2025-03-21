use reqwest::Client;
use serde_json::from_str;
use std::{
    env,
    error::Error,
    io::{Write, stdout},
};

use crate::{
    conversation::{ConversationContext, Response},
    spinner::run_with_spinner,
};

const API_URL: &str = "https://api.openai.com/v1/responses";

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
