use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use std::io::stdin;

pub async fn change_model_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
        let mut ctx = cc.conversation_context.lock().await;
        const AVAILABLE_MODELS: &[&str] = &[
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4o-search-preview",
            "o1",
            "o3-mini",
        ];

        println!("\nAvailable models:");
        for (i, model) in AVAILABLE_MODELS.iter().enumerate() {
            println!("{}) {}", i + 1, model);
        }
        println!("\nPlease select a model by typing its number:");
        let mut model_choice = String::new();
        stdin()
            .read_line(&mut model_choice)
            .expect("failed to read line");

        match model_choice.trim().parse::<usize>() {
            Ok(num) if num > 0 && num <= AVAILABLE_MODELS.len() => {
                ctx.model = AVAILABLE_MODELS[num - 1].to_string();
                println!("Model changed to: {}\n", ctx.model);
            }
            _ => eprintln!("Invalid selection. Keeping current model: {}\n", ctx.model),
        }
    }
    Ok(())
}
