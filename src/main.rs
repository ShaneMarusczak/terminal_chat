mod chat_client;
mod commands;
mod conversation;
mod messages;
mod spinner;

use chat_client::ChatClient;
use commands::handle_command;
use conversation::{ConversationContext, Message};
use messages::MESSAGES;
use rustyline::error::ReadlineError;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    println!("\n  -- terminal chat -- \n");

    let mut conversation_context = ConversationContext::new("gpt-4o");

    let developer_message = Message {
        role: "developer".into(),
        content: MESSAGES.get("developer").unwrap().to_string(),
    };

    conversation_context
        .messages
        .push(developer_message.clone());

    let chat_client = ChatClient::new()?;
    let mut rl = rustyline::DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline(">> ");
        let line = match readline {
            Ok(l) => l.trim().to_owned(),
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        };
        if line.is_empty() {
            continue;
        }
        rl.add_history_entry(&line).unwrap();
        if let Some(stripped) = line.strip_prefix(':') {
            let new: String = stripped.chars().filter(|c| !c.is_whitespace()).collect();
            let as_str = new.as_str();
            match as_str {
                "q" | "quit" => break,
                _ => {
                    handle_command(
                        as_str,
                        &mut conversation_context,
                        &developer_message,
                        &chat_client,
                    )
                    .await?
                }
            }
        } else {
            conversation_context.messages.push(Message {
                role: "user".into(),
                content: line.to_string(),
            });
            let response = chat_client.send_request(&conversation_context).await?;
            if let Some(choice) = response.choices.first() {
                let reply = choice.message.content.clone();
                conversation_context.messages.push(Message {
                    role: "assistant".into(),
                    content: reply.clone(),
                });
                println!("\n{}\n", reply);
            }
        }
    }
    Ok(())
}
