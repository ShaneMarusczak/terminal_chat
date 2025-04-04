use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    conversation::{ConversationContext, Message},
    tc_config::ConfigTC,
};

#[derive(Clone)]
pub struct CommandContext<'a> {
    pub conversation_context: Arc<Mutex<ConversationContext>>,
    pub dev_message: Arc<Message>,
    pub cmd: String,
    pub args: Vec<String>,
    pub config: &'a ConfigTC,
}

impl<'a> CommandContext<'a> {
    pub fn new(
        conversation_context: Arc<Mutex<ConversationContext>>,
        dev_message: Arc<Message>,
        cmd: String,
        args: Vec<String>,
        config: &'a ConfigTC,
    ) -> Self {
        Self {
            conversation_context,
            dev_message,
            cmd,
            args,
            config,
        }
    }
}
