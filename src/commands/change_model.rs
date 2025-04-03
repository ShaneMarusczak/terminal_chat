use serde::Deserialize;

use crate::chat_client::get_models;
use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use std::io::stdin;

pub(crate) const AVAILABLE_MODELS: &[&str] = &[
    "gpt-4o",
    "gpt-4o-mini",
    "gpt-4o-search-preview",
    "o1",
    "o3-mini",
];

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

        let models_response: ModelsResponse = serde_json::from_str(&get_models().await?)?;

        let names: Vec<String> = models_response.data.into_iter().map(|m| m.id).collect();
        let all_models: Vec<String> = AVAILABLE_MODELS
            .iter()
            .map(|&model| model.to_string())
            .chain(names)
            .collect();

        println!("\nAvailable models:");

        for (i, model) in all_models.iter().enumerate() {
            println!("{}) {}", i + 1, model);
        }
        println!("\nPlease select a model by typing its number:");
        let mut model_choice = String::new();
        stdin()
            .read_line(&mut model_choice)
            .expect("failed to read line");

        match model_choice.trim().parse::<usize>() {
            Ok(num) if num > 0 && num <= all_models.len() => {
                ctx.model = all_models[num - 1].to_string();
                println!("Model changed to: {}\n", ctx.model);
            }
            _ => eprintln!("Invalid selection. Keeping current model: {}\n", ctx.model),
        }
    }
    Ok(())
}
