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
        let config = get_config()?;

        let content_to_yank = if cc.args.is_empty() {
            // No args: yank the yank_target (last AI response or shown message)
            ctx.yank_target.clone()
        } else {
            // With arg: yank specific message by number
            let message_num: usize = cc.args[0].parse()
                .map_err(|_| "Invalid message number")?;

            if message_num >= ctx.input.len() {
                let error_msg = format!("Message {} not found. Valid range: 0-{}",
                    message_num, ctx.input.len() - 1);
                print_message(&error_msg, MessageType::System, &config);
                return Ok(());
            }

            let message = &ctx.input[message_num];
            if message.role == "developer" {
                print_message("Cannot yank developer/system messages", MessageType::System, &config);
                return Ok(());
            }

            Some(message.content.clone())
        };

        if let Some(content) = content_to_yank {
            let mut clipboard = Clipboard::new()?;
            clipboard.set_text(&content)?;
            print_message("Copied to clipboard", MessageType::System, &config);
        } else {
            print_message("Nothing to yank", MessageType::System, &config);
        }
    }
    Ok(())
}
