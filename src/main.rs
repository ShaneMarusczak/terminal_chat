use linefeed::{DefaultTerminal, Interface, ReadResult, complete::PathCompleter};
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

mod chat_client;
mod commands;
mod commands_registry;
mod conversation;
mod messages;
mod spinner;

use chat_client::ChatClient;
use commands::handle_command;
use conversation::{ConversationContext, Message};
use messages::MESSAGES;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("\n-- terminal chat -- \n");

    let context = Arc::new(Mutex::new(ConversationContext::new("gpt-4o-mini", true)));
    let dev_message = Arc::new(Message {
        role: "developer".into(),
        content: MESSAGES["developer"].to_string(),
    });
    let chat_client = Arc::new(ChatClient::new()?);
    let interface = build_interface()?;

    {
        let mut locked = context.lock().await;
        locked.input.push((*dev_message).clone());
    }

    while let ReadResult::Input(line) = interface.read_line()? {
        if line.trim().is_empty() {
            continue;
        }
        interface.add_history(line.clone());

        if let Some(cmd) = line.strip_prefix(':') {
            match cmd {
                "q" | "quit" => break,
                _ => {
                    handle_command(
                        cmd,
                        Arc::clone(&context),
                        Arc::clone(&dev_message),
                        Arc::clone(&chat_client),
                    )
                    .await?;
                }
            }
        } else {
            actually_chat(line, Arc::clone(&context), Arc::clone(&chat_client)).await?;
        }
    }
    Ok(())
}

fn build_interface() -> Result<Interface<DefaultTerminal>, Box<dyn Error>> {
    let interface = Interface::new("terminal chat interface")?;
    interface.set_completer(Arc::new(PathCompleter));
    interface.set_prompt("🗣️ ")?;
    Ok(interface)
}

async fn actually_chat(
    line: String,
    context: Arc<Mutex<ConversationContext>>,
    client: Arc<ChatClient>,
) -> Result<(), Box<dyn Error>> {
    {
        let mut locked = context.lock().await;
        locked.input.push(Message {
            role: "user".into(),
            content: line.clone(),
        });
    }

    let model = {
        let locked = context.lock().await;
        locked.model.clone()
    };

    if model.eq_ignore_ascii_case("gpt-4o-search-preview") {
        {
            let mut locked = context.lock().await;
            locked.set_stream(false);
        }
        let ctx = &context.lock().await;
        let response = client.send_request_c(ctx).await?;
        if let Some(choice) = response.choices.first() {
            let reply = choice.message.content.clone();
            {
                let mut locked = context.lock().await;
                locked.input.push(Message {
                    role: "assistant".into(),
                    content: reply.clone(),
                });
            }
            println!("\n🤖 {}\n", reply);
        }
        {
            let mut locked = context.lock().await;
            locked.set_stream(true);
        }
    } else {
        let mut ctx = context.lock().await;
        client.stream(&mut ctx).await?;
    }

    Ok(())
}
