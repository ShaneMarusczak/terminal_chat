use crate::commands::command_context::CommandContext;

use super::CommandResult;

pub async fn debug_command(cc: CommandContext) -> CommandResult {
    let ctx = cc.conversation_context.lock().await;
    println!("\nCurrent model: {}", ctx.model);
    println!("\nCurrent context messages:\n");
    for msg in &ctx.input {
        println!("{}:\n{}\n:::\n", msg.role, msg.content);
    }
    println!();
    Ok(())
}
