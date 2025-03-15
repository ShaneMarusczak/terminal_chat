use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Debug)]
pub struct ConversationContext {
    pub model: String,
    pub messages: Vec<Message>,
}

impl ConversationContext {
    pub fn new(model: &str) -> Self {
        Self {
            model: model.into(),
            messages: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    pub message: Message,
}
