use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Debug)]
pub struct ConversationContext {
    pub model: String,
    pub input: Vec<Message>,
}

impl ConversationContext {
    pub fn new(model: &str) -> Self {
        Self {
            model: model.into(),
            input: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub output: Vec<Output>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    #[serde(rename = "type")]
    pub type_field: String,
    pub id: String,
    pub status: String,
    pub role: String,
    pub content: Vec<OutputContent>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutputContent {
    #[serde(rename = "type")]
    pub type_field: String,
    pub text: String,
    pub annotations: Vec<serde_json::Value>,
}
