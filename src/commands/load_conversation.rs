use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use crate::conversation::ConversationContext;
use crate::utils::read_user_input;
use std::fs;

pub async fn lc_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
        let mut ctx = cc.conversation_context.lock().await;

        let convo_name = read_user_input("\nProvide conversation name: ");

        let as_str = fs::read_to_string(format!("conversations/{convo_name}.json"))?;

        let new_context: ConversationContext = serde_json::from_str(&as_str)?;

        *ctx = new_context;
    }
    Ok(())
}
