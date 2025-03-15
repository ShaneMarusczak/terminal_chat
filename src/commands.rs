use crate::chat_client::ChatClient;
use crate::conversation::{ConversationContext, Message};
use std::io::stdin;

const AVAILABLE_MODELS: &[&str] = &["gpt-4o", "gpt-4o-search-preview", "o1", "o3-mini"];

pub async fn handle_command(
    line: &str,
    context: &mut ConversationContext,
    dev_message: &Message,
    chat_client: &ChatClient,
) -> Result<bool, Box<dyn std::error::Error>> {
    if let Some(cmd) = line.strip_prefix(':') {
        match cmd {
            "q" => return Ok(true),
            "clear" => {
                context.messages.clear();
                context.messages.push(dev_message.clone());
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
                stdin().read_line(&mut model_choice)?;
                let index: usize = match model_choice.trim().parse() {
                    Ok(num) if num > 0 && num <= AVAILABLE_MODELS.len() => num,
                    _ => {
                        println!(
                            "Invalid selection. Keeping current model: {}",
                            context.model
                        );
                        return Ok(false);
                    }
                };
                context.model = AVAILABLE_MODELS[index - 1].to_string();
                println!("Model changed to '{}'\n", context.model);
            }
            _ => eprintln!("\nInvalid command: {}\n", cmd),
        }
    } else {
        context.messages.push(Message {
            role: "user".into(),
            content: line.to_string(),
        });
        let response = chat_client.send_request(context).await?;
        if let Some(choice) = response.choices.first() {
            let reply = choice.message.content.clone();
            context.messages.push(Message {
                role: "assistant".into(),
                content: reply.clone(),
            });
            println!("\n{}\n", reply);
        }
    }
    Ok(false)
}
