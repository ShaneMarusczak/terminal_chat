use crate::commands::{command_context::CommandContext, commands_registry::TC_COMMANDS};

use crate::commands::command_tc::CommandResult;
use crate::message_printer::{MessageType, print_message};
use crate::tc_config::get_config;

pub async fn help_command(_cc: Option<CommandContext>) -> CommandResult {
    let mut output = String::new();
    output.push_str("\nAvailable commands:\n");

    for tc in TC_COMMANDS.values() {
        if tc.name.len() < 7 {
            output.push_str(&format!("{:7} - {}\n", tc.name, tc.description));
        } else {
            let gap = " ".repeat(7);
            output.push_str(&format!("{}\n{} - {}\n", tc.name, gap, tc.description));
        }
    }

    print_message(&output, MessageType::System, &get_config()?);
    Ok(())
}
