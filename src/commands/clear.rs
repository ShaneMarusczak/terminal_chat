use crate::commands::command_context::CommandContext;
use std::process::Command;

use super::CommandResult;

pub async fn clear_command(cc: CommandContext) -> CommandResult {
    {
        let mut ctx = cc.conversation_context.lock().await;
        ctx.input.clear();
        ctx.input.push((*cc.dev_message).clone());
    } // release lock

    Command::new("clear")
        .status()
        .expect("clear command failed");
    println!("\nConversation cleared.\n");
    Ok(())
}
