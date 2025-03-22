mod chat_client;
mod commands;
mod conversation;
mod messages;
mod spinner;

use chat_client::ChatClient;
use commands::handle_command;
use conversation::{ConversationContext, Message};
use linefeed::{DefaultTerminal, Interface, ReadResult, complete::PathCompleter};
use messages::MESSAGES;
use std::{error::Error, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("\n-- terminal chat -- \n");

    let mut conversation_context = ConversationContext::new("gpt-4o", true);
    let developer_message = Message {
        role: "developer".into(),
        content: MESSAGES["developer"].to_string(),
    };
    conversation_context.input.push(developer_message.clone());

    let chat_client = ChatClient::new()?;
    let interface = build_interface()?;

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
                        &mut conversation_context,
                        &developer_message,
                        &chat_client,
                    )
                    .await?
                }
            }
        } else {
            actually_chat(line, &mut conversation_context, &chat_client).await?;
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
    context: &mut ConversationContext,
    client: &ChatClient,
) -> Result<(), Box<dyn Error>> {
    context.input.push(Message {
        role: "user".into(),
        content: line.clone(),
    });

    if context.model.eq_ignore_ascii_case("gpt-4o-search-preview") {
        context.set_stream(false);
        let response = client.send_request_c(context).await?;

        if let Some(choice) = response.choices.first() {
            let reply = choice.message.content.clone();
            context.input.push(Message {
                role: "assistant".into(),
                content: reply.clone(),
            });
            println!("\n🤖 {}\n", reply);
        }
        context.set_stream(true);
    } else {
        client.stream(context).await?;
    }

    Ok(())
}
