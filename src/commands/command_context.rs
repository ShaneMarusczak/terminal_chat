use std::sync::Arc;
use tokio::sync::Mutex;

use crate::conversation::{ConversationContext, Message};

#[derive(Clone)]
pub struct CommandContext {
    pub conversation_context: Arc<Mutex<ConversationContext>>,
    pub dev_message: Arc<Message>,
    pub cmd: String,
    pub args: Vec<String>,
}

impl CommandContext {
    pub fn new(
        conversation_context: Arc<Mutex<ConversationContext>>,
        dev_message: Arc<Message>,
        cmd: String,
        args: Vec<String>,
    ) -> Self {
        Self {
            conversation_context,
            dev_message,
            cmd,
            args,
        }
    }
}
