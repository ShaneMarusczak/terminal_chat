use crate::commands::commands_registry::TC_COMMANDS;
use crate::commands::handle_commands::handle_command;
use linefeed::{DefaultTerminal, Interface, ReadResult, complete::PathCompleter};
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

use crate::chat_client::ChatClient;
use crate::conversation::{ConversationContext, Message, ResponseC};
use crate::messages::MESSAGES;

pub(crate) async fn as_repl() -> Result<(), Box<dyn Error>> {
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
                    if let Err(e) = handle_command(
                        cmd,
                        Arc::clone(&context),
                        Arc::clone(&dev_message),
                        Arc::clone(&chat_client),
                    )
                    .await
                    {
                        eprintln!("Error executing command: {} With error: {}", cmd, e);
                    }
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
    let mut ctx = context.lock().await;

    ctx.input.push(Message {
        role: "user".into(),
        content: line.clone(),
    });

    if ctx.model.eq_ignore_ascii_case("gpt-4o-search-preview") {
        ctx.set_stream(false);
        let response: ResponseC = client.send_request("chat", &*ctx).await?;
        if let Some(choice) = response.choices.first() {
            let reply = choice.message.content.clone();
            {
                ctx.input.push(Message {
                    role: "assistant".into(),
                    content: reply.clone(),
                });
            }
            println!("\n🤖 {}\n", reply);
        }
        ctx.set_stream(true);
    } else {
        client.stream(&mut ctx).await?;
    }

    Ok(())
}

pub(crate) async fn as_cli_tool(args: &[String]) -> Result<(), Box<dyn Error>> {
    match args.len() {
        1 => match args[0].as_str() {
            "-h" | "--help" => {
                if let Some(help_command) = TC_COMMANDS.get("help") {
                    return (help_command.run)(None).await; // Call help command
                } else {
                    eprintln!("Help command not found!");
                }
            }
            _ => {
                // Handle other commands or show some usage message
            }
        },
        _ => {
            // Handle cases where args.len() is not 1
        }
    }

    Ok(())
}
