use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use crate::conversation::ConversationContext;
use crate::utils::read_user_input;
use std::fs;

pub async fn lc_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
        let mut ctx = cc.conversation_context.lock().await;

        let convo_name = read_user_input("\nProvide conversation name: ")?;

        let mut path = dirs::home_dir().ok_or("Could not find home directory")?;
        path.push(".tc");
        path.push("conversations");
        path.push(format!("{convo_name}.json"));

        let as_str = fs::read_to_string(&path)
            .map_err(|e| format!("Could not read '{}': {}", path.display(), e))?;

        let new_context: ConversationContext = serde_json::from_str(&as_str)?;
        *ctx = new_context;

        println!("Conversation loaded from: {}", path.display());
    }
    Ok(())
}
