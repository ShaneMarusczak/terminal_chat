use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use crate::tc_config::{GLOBAL_CONFIG, get_config, write_config};
use crate::utils::read_user_input;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Model {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<Model>,
}

pub async fn change_model_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
        let mut ctx = cc.conversation_context.lock().await;
        let config = get_config();

        println!("\nAvailable models:");

        for (i, model) in config.all_models.iter().enumerate() {
            println!("{}) {}", i + 1, model);
        }

        let model_choice = read_user_input("Please select a model by entering its number:");

        match model_choice.trim().parse::<usize>() {
            Ok(num) if num > 0 && num <= config.all_models.len() => {
                ctx.model = config.all_models[num - 1].to_string();
                {
                    let mut cg = GLOBAL_CONFIG.write().unwrap();
                    cg.model = ctx.model.clone();
                }
                write_config(&get_config(), false)?;
                println!("Model changed to: {}\n", ctx.model);
            }
            _ => eprintln!("Invalid selection. Keeping current model: {}\n", ctx.model),
        }
    }
    Ok(())
}
