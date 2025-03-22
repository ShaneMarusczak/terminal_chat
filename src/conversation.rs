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
    pub stream: bool,
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
