use crate::{
    commands::command_context::CommandContext,
    commands::command_tc::CommandResult,
    message_printer::{MessageType, print_message},
    tc_config::get_config,
};
use std::fs;

pub async fn fork_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
        let ctx = cc.conversation_context.lock().await;
        let config = get_config()?;

        // Create conversations directory if it doesn't exist
        let mut path = dirs::home_dir().ok_or("Could not find home directory")?;
        path.push(".tc");
        path.push("conversations");
        fs::create_dir_all(&path)?;

        // Generate fork filename with timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let fork_name = format!("fork-{}.json", timestamp);
        path.push(&fork_name);

        // Save the current conversation
        let json = serde_json::to_string_pretty(&*ctx)?;
        fs::write(&path, json)?;

        let message = format!("Conversation forked to: {}", fork_name);
        print_message(&message, MessageType::System, &config);
    }
    Ok(())
}
