use std::sync::Arc;
use tokio::sync::Mutex;

use crate::conversation::{ConversationContext, Message};

#[derive(Clone)]
pub struct CommandContext {
    pub conversation_context: Arc<Mutex<ConversationContext>>,
    pub dev_message: Arc<Message>,
    pub openai_enabled: bool,
    pub anthropic_enabled: bool,
    pub cmd: String,
    pub args: Vec<String>,
}

impl CommandContext {
    pub fn new(
        conversation_context: Arc<Mutex<ConversationContext>>,
        dev_message: Arc<Message>,
        openai_enabled: bool,
        anthropic_enabled: bool,
        cmd: String,
        args: Vec<String>,
    ) -> Self {
        Self {
            conversation_context,
            dev_message,
            anthropic_enabled,
            openai_enabled,
            cmd,
            args,
        }
    }
}
