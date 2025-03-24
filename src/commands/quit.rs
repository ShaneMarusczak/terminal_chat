use crate::commands::command_context::CommandContext;

use super::CommandResult;

pub async fn quit_command(_cc: CommandContext) -> CommandResult {
    unreachable!()
}
