use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    chat_client::ChatClient,
    conversation::{ConversationContext, Message},
};

#[derive(Clone)]
pub struct CommandContext {
    pub conversation_context: Arc<Mutex<ConversationContext>>,
    pub dev_message: Arc<Message>,
    pub chat_client: Arc<ChatClient>,
    pub cmd: String,
    pub args: Vec<String>,
}

impl CommandContext {
    pub fn new(
        conversation_context: Arc<Mutex<ConversationContext>>,
        dev_message: Arc<Message>,
        chat_client: Arc<ChatClient>,
        cmd: String,
        args: Vec<String>,
    ) -> Self {
        Self {
            conversation_context,
            dev_message,
            chat_client,
            cmd,
            args,
        }
    }
}
