use crate::{
    commands::command_context::CommandContext,
    commands::command_tc::CommandResult,
    message_printer::{MessageType, print_message},
    tc_config::get_config,
};
use arboard::Clipboard;

pub async fn yank_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
        let ctx = cc.conversation_context.lock().await;

        if let Some(last_response) = &ctx.last_response {
            let mut clipboard = Clipboard::new()?;
            clipboard.set_text(last_response)?;
            print_message("Last response copied to clipboard", MessageType::System, &get_config()?);
        } else {
            print_message("No response to yank", MessageType::System, &get_config()?);
        }
    }
    Ok(())
}
