use crate::chat_client::{anthropic_chat, send_request, stream};
use crate::commands::commands_registry::TC_COMMANDS;
use crate::commands::handle_commands::handle_command;
use crate::conversation::{AnthropicMessage, ConversationContext, Message, ResponseC};
use crate::message_printer::{MessageType, print_message};
use crate::preview_md::markdown_to_ansi;
use crate::tc_config::{self, get_config};
use crate::utils::calculate_message_width;
use linefeed::{DefaultTerminal, Interface, ReadResult, complete::PathCompleter};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) async fn as_repl() -> Result<(), Box<dyn Error>> {
    let config = tc_config::load_config().await?;
    print_message("~~~  Terminal Chat  ~~~", MessageType::System, &config);

    if !config.openai_enabled && !config.anthropic_enabled {
        return Ok(());
    }

    let context = Arc::new(Mutex::new(ConversationContext::new(
        &config.model,
        config.enable_streaming,
    )));

    let dev_message = Arc::new(Message {
        role: "developer".into(),
        content: config.dev_message.clone(),
    });
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
                    if let Err(e) =
                        handle_command(cmd, Arc::clone(&context), Arc::clone(&dev_message)).await
                    {
                        eprintln!("Error executing command: {} With error: {}", cmd, e);
                    }
                }
            }
        } else {
            actually_chat(line, Arc::clone(&context)).await?;
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
) -> Result<(), Box<dyn Error>> {
    let mut ctx = context.lock().await;
    let config = get_config()?;
    if !config.enable_streaming && config.message_boxes_enabled {
        let (width, terminal_width) = calculate_message_width(&line, 70, 80);

        let width = width.min(terminal_width);

        let line_len = line.chars().count();
        let line_count = (line_len / width) + if line_len % width == 0 { 0 } else { 1 };

        // Clear previous lines
        for _ in 0..line_count {
            print!("\x1B[1A\x1B[2K");
        }
        print_message(&line, MessageType::User, &config);
    }

    ctx.input.push(Message {
        role: "user".into(),
        content: line.clone(),
    });

    if ctx.model.contains("claude") {
        ctx.set_stream(false);

        let reply: AnthropicMessage = anthropic_chat(&ctx).await?;

        let message = reply.content.first().ok_or("No content")?.text.clone();

        if config.message_boxes_enabled {
            print_message(&message, MessageType::Assistant, &config);
            println!();
        } else {
            println!("🤖 {}\n", message);
        }
        ctx.input.push(Message {
            role: "assistant".into(),
            content: message.clone(),
        });

        ctx.set_stream(true);
    } else if !config.enable_streaming || ctx.model.eq_ignore_ascii_case("gpt-4o-search-preview") {
        ctx.set_stream(false);
        let response: ResponseC = send_request("chat", &*ctx).await?;
        if let Some(choice) = response.choices.first() {
            let reply = choice.message.content.clone();
            ctx.input.push(Message {
                role: "assistant".into(),
                content: reply.clone(),
            });

            let s = if config.preview_md {
                markdown_to_ansi(&reply)
            } else {
                reply
            };

            if config.message_boxes_enabled {
                print_message(&s, MessageType::Assistant, &config);
                println!();
            } else {
                println!("\n🤖 {}", s);
            }
        }
        ctx.set_stream(true);
    } else {
        stream(&mut ctx).await?;
    }

    Ok(())
}

pub(crate) async fn as_cli_tool(args: &[String]) -> Result<(), Box<dyn Error>> {
    match args.len() {
        1 => match args[0].as_str() {
            "-h" | "--help" => {
                if let Some(help_command) = TC_COMMANDS.get("help") {
                    return (help_command.run)(None).await;
                } else {
                    eprintln!("Help command not found!");
                }
            }
            _ => {
                // Handle other commands or show some usage message
            }
        },
        _ => {
            // make readme a cli command that takes the same args as in the repl
            // dir and extension
            //
            // summarize? takes any single file?
            //
            //
            // Handle cases where args.len() is not 1
        }
    }

    Ok(())
}
