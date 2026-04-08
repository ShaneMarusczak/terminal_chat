use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Anthropic,
    OpenAI,
    Local,
}

impl Provider {
    /// Determines the provider based on the model name.
    /// `local/` prefix routes to the local endpoint.
    /// Names containing `claude` route to Anthropic.
    /// Everything else routes to OpenAI.
    pub fn from_model_name(model: &str) -> Self {
        let model_lower = model.to_lowercase();
        if model_lower.starts_with("local/") {
            Provider::Local
        } else if model_lower.contains("claude") {
            Provider::Anthropic
        } else {
            Provider::OpenAI
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Developer,
    System,
    User,
    Assistant,
}

impl Role {
    pub fn display_name(self) -> &'static str {
        match self {
            Role::User => "You",
            Role::Assistant => "AI",
            Role::Developer | Role::System => "System",
        }
    }

    pub fn is_visible(self) -> bool {
        matches!(self, Role::User | Role::Assistant)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConversationContext {
    pub model: String,
    pub input: Vec<Message>,
    #[serde(skip)]
    pub yank_target: Option<String>,
}

impl ConversationContext {
    pub fn new(model: &str) -> Self {
        Self {
            model: model.into(),
            input: Vec::new(),
            yank_target: None,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
}

impl OpenAIRequest {
    /// Build an OpenAI-compatible request. Maps Developer → System role
    /// so the system prompt reaches the model.
    pub fn new(model: &str, ctx: &ConversationContext, max_tokens: Option<usize>) -> Self {
        Self {
            model: model.to_string(),
            messages: ctx
                .input
                .iter()
                .map(|m| {
                    if m.role == Role::Developer {
                        Message {
                            role: Role::System,
                            content: m.content.clone(),
                        }
                    } else {
                        m.clone()
                    }
                })
                .collect(),
            max_tokens,
        }
    }
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
            .find(|m| m.role == Role::Developer)
            .map(|m| m.content.clone())
            .unwrap_or_default();
        Self {
            system: system_content,
            model: ctx.model.clone(),
            max_tokens,
            messages: ctx
                .input
                .iter()
                .filter(|m| m.role != Role::Developer)
                .cloned()
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseC {
    pub choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    pub message: Message,
}
