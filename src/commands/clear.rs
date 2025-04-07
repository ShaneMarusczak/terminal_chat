use crate::{
    commands::command_context::CommandContext,
    message_printer::{MessageType, print_message},
    tc_config::get_config,
};
use std::process::Command;

use crate::commands::command_tc::CommandResult;

pub async fn clear_command(cc: Option<CommandContext>) -> CommandResult {
    {
        if let Some(cc) = cc {
            let mut ctx = cc.conversation_context.lock().await;
            ctx.input.clear();
            ctx.input.push((*cc.dev_message).clone());
        }

        Command::new("clear").status()?;

        print_message("Conversation cleared", MessageType::System, &get_config()?);
    }
    Ok(())
}
