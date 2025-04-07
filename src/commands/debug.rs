use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use crate::tc_config::{get_config, print_config};

pub async fn debug_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
        let ctx = cc.conversation_context.lock().await;
        println!("\nCurrent model: {}", ctx.model);
        println!("\nCurrent context messages:\n");
        for msg in &ctx.input {
            println!("{}:\n{}\n:::\n", msg.role, msg.content);
        }

        // Get the current config
        let config = get_config()?;
        print_config(&config);
        println!();
    }
    Ok(())
}
