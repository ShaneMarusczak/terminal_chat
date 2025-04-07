use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use crate::conversation::Message;
use crate::message_printer::{MessageType, print_message};
use crate::tc_config::get_config;
use std::fs;
use std::path::Path;

pub async fn gf_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
        let mut ctx = cc.conversation_context.lock().await;
        if cc.args.is_empty() {
            eprintln!(
                "\nInvalid use of {}. Usage: {} <path1> <path2> ...\n",
                cc.cmd, cc.cmd
            );
            return Ok(());
        }

        for path in cc.args {
            let trimmed_path = path.trim();
            match fs::read_to_string(Path::new(trimmed_path)) {
                Ok(content) => {
                    ctx.input.push(Message {
                        role: "user".to_string(),
                        content: format!("{}\n\n:::\n\n{}", trimmed_path, content),
                    });
                    let message = format!("Added: {trimmed_path}");
                    print_message(&message, MessageType::System, &get_config()?);
                }
                Err(e) => eprintln!("Error reading {}: {}\n", trimmed_path, e),
            }
        }
    }
    Ok(())
}
