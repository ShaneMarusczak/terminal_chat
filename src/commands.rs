use crate::chat_client::ChatClient;
use crate::conversation::{ConversationContext, Message};
use std::io::stdin;
use std::process::Command;

const AVAILABLE_MODELS: &[&str] = &["gpt-4o", "gpt-4o-search-preview", "o1", "o3-mini"];

pub async fn handle_command(
    cmd: &str,
    context: &mut ConversationContext,
    dev_message: &Message,
    chat_client: &ChatClient,
) -> Result<(), reqwest::Error> {
    match cmd {
        "c" | "clear" => {
            context.messages.clear();
            context.messages.push(dev_message.clone());
            Command::new("clear")
                .status()
                .expect("clear command failed");
            println!("\nConversation cleared.\n");
        }
        "debug" => {
            println!("\nDebugging conversation messages:");
            for msg in &context.messages {
                println!("{}: {}", msg.role, msg.content);
            }
            println!();
        }
        "doc" => {
            chat_client.document(context).await?;
        }
        "cm" => {
            println!("\nAvailable models:");
            for (i, model) in AVAILABLE_MODELS.iter().enumerate() {
                println!("{}) {}", i + 1, model);
            }
            println!("Please select a model by typing its number (e.g., 1 or 2):");
            let mut model_choice = String::new();
            stdin()
                .read_line(&mut model_choice)
                .expect("failed to read line");
            let index: usize = match model_choice.trim().parse() {
                Ok(num) if num > 0 && num <= AVAILABLE_MODELS.len() => num,
                _ => {
                    eprintln!(
                        "Invalid selection. Keeping current model: {}",
                        context.model
                    );
                    return Ok(());
                }
            };
            context.model = AVAILABLE_MODELS[index - 1].to_string();
            println!("Model changed to '{}'\n", context.model);
        }
        _ => eprintln!("\nInvalid command: {}\n", cmd),
    }
    Ok(())
}
