use super::CommandResult;
use crate::commands::command_context::CommandContext;
use std::process::Command;

pub async fn sh(cc: CommandContext) -> CommandResult {
    if cc.args.is_empty() {
        eprintln!("\nUsage: sh <program> [args...]\n");
        return Ok(());
    }

    let program = &cc.args[0];
    let program_args = &cc.args[1..]; // empty slice if no further arguments

    Command::new(program).args(program_args).status()?;

    Ok(())
}
