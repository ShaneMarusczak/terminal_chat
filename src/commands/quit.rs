use crate::commands::command_context::CommandContext;

use crate::commands::command_tc::CommandResult;

pub async fn quit_command(_cc: Option<CommandContext<'_>>) -> CommandResult {
    unreachable!()
}
