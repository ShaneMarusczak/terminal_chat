use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use serde::Deserialize;
use std::io::stdin;

#[derive(Debug, Deserialize)]
pub struct Model {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<Model>,
}

pub async fn change_model_command(cc: Option<CommandContext<'_>>) -> CommandResult {
    if let Some(cc) = cc {
        let mut ctx = cc.conversation_context.lock().await;

        println!("\nAvailable models:");

        for (i, model) in cc.config.all_models.iter().enumerate() {
            println!("{}) {}", i + 1, model);
        }
        println!("\nPlease select a model by typing its number:");
        let mut model_choice = String::new();
        stdin()
            .read_line(&mut model_choice)
            .expect("failed to read line");

        match model_choice.trim().parse::<usize>() {
            Ok(num) if num > 0 && num <= cc.config.all_models.len() => {
                ctx.model = cc.config.all_models[num - 1].to_string();
                println!("Model changed to: {}\n", ctx.model);
            }
            _ => eprintln!("Invalid selection. Keeping current model: {}\n", ctx.model),
        }
    }
    Ok(())
}
