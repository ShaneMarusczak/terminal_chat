use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use crate::conversation::{ConversationContext, Message, Role};
use crate::message_printer::print_message;
use crate::tc_config::get_config;
use std::fs;
use std::path::Path;

/// Load files into a conversation context. Returns the list of paths that were
/// successfully added. Errors for individual files are printed to stderr so a
/// single bad path doesn't abort the rest of the batch.
pub fn load_files(ctx: &mut ConversationContext, paths: &[String]) -> Vec<String> {
    let mut added = Vec::new();
    for path in paths {
        let trimmed = path.trim();
        match fs::read_to_string(Path::new(trimmed)) {
            Ok(content) => {
                ctx.input.push(Message {
                    role: Role::User,
                    content: format!("{}\n\n:::\n\n{}", trimmed, content),
                });
                added.push(trimmed.to_string());
            }
            Err(e) => eprintln!("Error reading {}: {}", trimmed, e),
        }
    }
    added
}

pub async fn gf_command(cc: Option<CommandContext>) -> CommandResult {
    let Some(cc) = cc else { return Ok(()) };

    if cc.args.is_empty() {
        eprintln!(
            "\nInvalid use of {}. Usage: {} <path1> <path2> ...\n",
            cc.cmd, cc.cmd
        );
        return Ok(());
    }

    let added = {
        let mut ctx = cc.conversation_context.lock().await;
        load_files(&mut ctx, &cc.args)
    };

    let config = get_config()?;
    for path in added {
        print_message(&format!("Added: {path}"), Role::System, &config);
    }
    Ok(())
}
