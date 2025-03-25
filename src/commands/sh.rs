use super::CommandResult;
use crate::commands::command_context::CommandContext;
use std::process::Command;

pub async fn sh(cc: CommandContext) -> CommandResult {
    // Expecting cc.args: ["program", "arg1", "arg2", ...]
    if cc.args.is_empty() {
        eprintln!("Usage: sh <program> [args...]");
        return Ok(());
    }

    let program = &cc.args[0];
    let program_args = &cc.args[1..]; // empty slice if no further arguments

    println!();
    match Command::new(program).args(program_args).status() {
        Ok(status) => println!("Process exited with status: {}", status),
        Err(err) => eprintln!("Failed to execute '{}': {}", program, err),
    }
    println!();

    Ok(())
}
