use crate::{
    commands::command_context::CommandContext,
    commands::command_tc::CommandResult,
    conversation::Role,
    message_printer::print_message,
    tc_config::get_config,
};
use std::fs;

pub async fn fork_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
        let ctx = cc.conversation_context.lock().await;
        let config = get_config()?;

        let mut path = dirs::home_dir().ok_or("Could not find home directory")?;
        path.push(".tc");
        path.push("conversations");
        fs::create_dir_all(&path)?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let fork_name = format!("fork-{}.json", timestamp);
        path.push(&fork_name);

        let json = serde_json::to_string_pretty(&*ctx)?;
        fs::write(&path, json)?;

        let message = format!("Conversation forked to: {}", path.display());
        print_message(&message, Role::System, &config);
    }
    Ok(())
}
