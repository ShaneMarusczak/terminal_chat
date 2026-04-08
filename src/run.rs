use crate::chat_client::chat;
use crate::cli::{CliArgs, Mode, parse_cli_args};
use crate::commands::document::generate_document;
use crate::commands::gf::load_files;
use crate::commands::handle_commands::handle_command;
use crate::commands::readme::generate_readme;
use crate::conversation::{ConversationContext, Message, Role};
use crate::message_printer::print_message;
use crate::tc_config::{self, ConfigTC, get_config, load_config_file};
use linefeed::{DefaultTerminal, Interface, ReadResult, complete::PathCompleter};
use std::collections::HashSet;
use std::error::Error;
use std::io::{IsTerminal, Read, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// REPL mode
// ---------------------------------------------------------------------------

pub(crate) async fn as_repl() -> Result<(), Box<dyn Error>> {
    let config = tc_config::load_config().await?;
    println!("~~~  Terminal Chat  ~~~");

    if !config.openai_enabled && !config.anthropic_enabled {
        return Ok(());
    }

    let context = Arc::new(Mutex::new(ConversationContext::new(&config.model)));
    let dev_message = Arc::new(Message {
        role: Role::Developer,
        content: config.dev_message.clone(),
    });

    {
        let mut locked = context.lock().await;
        locked.input.push((*dev_message).clone());
    }

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

    ctx.input.push(Message {
        role: Role::User,
        content: line,
    });

    let reply = chat(&ctx, &config, None).await?;

    print_message(&reply, Role::Assistant, &config);
    println!();

    ctx.input.push(Message {
        role: Role::Assistant,
        content: reply.clone(),
    });
    ctx.yank_target = Some(reply);

    Ok(())
}

// ---------------------------------------------------------------------------
// CLI mode — Unix-style: args are the prompt, stdin is the content
// ---------------------------------------------------------------------------

pub(crate) async fn as_cli_tool(args: &[String]) -> Result<(), Box<dyn Error>> {
    let cli = parse_cli_args(args)?;

    if cli.help {
        print_cli_usage();
        return Ok(());
    }

    load_config_file()?;
    let config = get_config()?;
    let model = resolve_model(cli.model_override.as_deref(), cli.use_local, &config.model);

    match cli.mode {
        Mode::Chat => run_chat(cli, config, model).await,
        Mode::Doc => run_doc(cli, config, model).await,
        Mode::Readme => run_readme(cli, config, model).await,
    }
}

fn resolve_model(cli_model: Option<&str>, use_local: bool, default: &str) -> String {
    if let Some(m) = cli_model {
        m.to_string()
    } else if use_local {
        format!("local/{}", default)
    } else {
        default.to_string()
    }
}

async fn run_chat(cli: CliArgs, config: ConfigTC, model: String) -> Result<(), Box<dyn Error>> {
    let stdin_content = read_piped_stdin()?;

    let user_message: Option<String> = match (stdin_content, cli.prompt) {
        (Some(content), Some(prompt)) => Some(format!("{}\n\n---\n\n{}", content, prompt)),
        (Some(content), None) => Some(content),
        (None, Some(prompt)) => Some(prompt),
        (None, None) if cli.gf_files.is_empty() => {
            print_cli_usage();
            return Err("No prompt provided".into());
        }
        (None, None) => None,
    };

    let mut ctx = ConversationContext::new(&model);
    ctx.input.push(Message {
        role: Role::Developer,
        content: cli.system_override.unwrap_or_else(|| config.dev_message.clone()),
    });

    if !cli.gf_files.is_empty() {
        load_files(&mut ctx, &cli.gf_files);
    }

    if let Some(content) = user_message {
        ctx.input.push(Message {
            role: Role::User,
            content,
        });
    }

    let reply = chat(&ctx, &config, cli.max_tokens).await?;

    print!("{}", reply);
    std::io::stdout().flush()?;

    Ok(())
}

async fn run_doc(cli: CliArgs, config: ConfigTC, model: String) -> Result<(), Box<dyn Error>> {
    let mut messages: Vec<Message> = Vec::new();

    if !cli.gf_files.is_empty() {
        let mut tmp = ConversationContext::new(&model);
        load_files(&mut tmp, &cli.gf_files);
        messages.extend(tmp.input);
    }

    if let Some(content) = read_piped_stdin()? {
        messages.push(Message {
            role: Role::User,
            content,
        });
    }

    if let Some(prompt) = cli.prompt {
        messages.push(Message {
            role: Role::User,
            content: prompt,
        });
    }

    if messages.is_empty() {
        return Err("doc: no input. Use --gf <files>, pipe via stdin, or pass a prompt.".into());
    }

    let (title, report) = generate_document(&messages, &model, &config).await?;
    let file_contents = format!("{}\n\n{}", title, report);

    write_output(cli.output.as_deref(), &file_contents, "Document")
}

async fn run_readme(cli: CliArgs, config: ConfigTC, model: String) -> Result<(), Box<dyn Error>> {
    let dir = cli
        .directory
        .ok_or("readme: directory is required. Usage: tc readme <dir> [extensions...]")?;
    let extensions: HashSet<&str> = cli.extensions.iter().map(String::as_str).collect();

    let result = generate_readme(&dir, &extensions, &model, &config).await?;

    write_output(cli.output.as_deref(), &result, "README")
}

fn read_piped_stdin() -> Result<Option<String>, Box<dyn Error>> {
    if std::io::stdin().is_terminal() {
        return Ok(None);
    }
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    Ok((!buf.is_empty()).then_some(buf))
}

fn write_output(output_path: Option<&str>, content: &str, label: &str) -> Result<(), Box<dyn Error>> {
    if let Some(path) = output_path {
        let mut file = std::fs::File::create(path)
            .map_err(|e| format!("Could not create file '{}': {}", path, e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Could not write to file '{}': {}", path, e))?;
        eprintln!("{} saved to: {}", label, path);
    } else {
        print!("{}", content);
        std::io::stdout().flush()?;
    }
    Ok(())
}

const CLI_USAGE: &str = "tc - Terminal Chat

Usage:
  tc                                Start interactive REPL
  tc <prompt>                       One-shot prompt
  tc --gf file1 file2 <prompt>      Load files as context, then prompt
  cat file | tc <prompt>            Send piped content with prompt
  cat file | tc <prompt> > out.md   Pipe result to file

Subcommands:
  tc doc [--gf files...]            Generate document from stdin / files
  tc readme <dir> [exts...]         Generate README from a directory

Options:
  -m, --model <name>    Override model (e.g. local/qwen3.5)
  --local               Use local model endpoint
  --max-tokens <n>      Max response tokens
  -s, --system <text>   Override system prompt
  --gf <files...>       Load files into context
  -o, --output <path>   Write result to file (doc/readme)
  -h, --help            Show this help";

fn print_cli_usage() {
    eprintln!("{}", CLI_USAGE);
}
