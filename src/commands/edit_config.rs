use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use crate::tc_config::{GLOBAL_CONFIG, config_interview, get_config, write_config};

pub async fn ec_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
        let mut ctx = cc.conversation_context.lock().await;
        let mut config = get_config()?;

        config_interview(&mut config);

        {
            let mut config_guard = GLOBAL_CONFIG.write()?;
            *config_guard = config.clone();
        }
        if !ctx.input.is_empty() {
            for message in &mut ctx.input {
                if message.role == "developer" {
                    message.content = config.dev_message.clone();
                    break;
                }
            }
        }
        ctx.model = config.model.clone();
        write_config(&config, false)?;

        println!("Configuration updated successfully!");
    }
    Ok(())
}
