use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConversationContext {
    pub model: String,
    pub input: Vec<Message>,
    pub stream: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnthropicRequest {
    pub system: String,
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: usize,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicMessageContent {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicMessage {
    pub content: Vec<AnthropicMessageContent>,
}

impl AnthropicRequest {
    pub fn from_context(ctx: &ConversationContext, max_tokens: usize) -> Self {
        let system_content = ctx
            .input
            .iter()
            .find(|m| m.role == "developer")
            .map(|m| m.content.clone())
            .unwrap_or_default();
        Self {
            system: system_content,
            model: ctx.model.clone(),
            max_tokens,
            messages: ctx
                .input
                .iter()
                .filter(|m| m.role != "developer")
                .cloned()
                .collect(),
        }
    }
}

impl ConversationContext {
    pub fn new(model: &str, stream: bool) -> Self {
        Self {
            model: model.into(),
            input: Vec::new(),
            stream,
        }
    }

    pub fn set_stream(&mut self, s: bool) {
        self.stream = s;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub output: Vec<Output>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseC {
    pub choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    pub message: Message,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(rename = "type")]
    pub type_field: String,
    pub id: String,
    pub status: Option<String>,
    pub role: Option<String>,
    pub content: Option<Vec<OutputContent>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutputContent {
    #[serde(rename = "type")]
    pub type_field: String,
    pub text: String,
    pub annotations: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct DeltaData {
    pub delta: String,
}
