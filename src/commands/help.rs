use crate::commands::{command_context::CommandContext, commands_registry::TC_COMMANDS};

use crate::commands::command_tc::CommandResult;

pub async fn help_command(_cc: Option<CommandContext<'_>>) -> CommandResult {
    println!("\nAvailable commands:\n");
    for tc in TC_COMMANDS.values() {
        if tc.name.len() < 7 {
            println!("{:7} - {}", tc.name, tc.description);
        } else {
            let gap = " ".repeat(7);
            println!("{}", tc.name);
            println!("{gap} - {}", tc.description);
        }
    }
    println!();
    Ok(())
}
