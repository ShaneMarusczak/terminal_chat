use tokio::sync::Mutex;

use crate::{
    chat_client::ChatClient,
    conversation::{ConversationContext, Message, Response},
    messages::MESSAGES,
};
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::{self, File},
    future::Future,
    io::{Write, stdin},
    path::Path,
    pin::Pin,
    process::Command,
    sync::{Arc, LazyLock},
};

pub type CommandResult = Result<(), Box<dyn Error>>;

pub static TC_COMMANDS: LazyLock<HashMap<&str, CommandTC>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    //define commands
    let clear_cmd = CommandTC::new("clear", "Clears the conversation context.", |cc| {
        Box::pin(clear_command(cc))
    });
    let rmr_cmd = CommandTC::new("rmr", "Launches rmr if isntalled.", |cc| {
        Box::pin(start_rmr(cc))
    });
    let debug_cmd = CommandTC::new(
        "debug",
        "Prints debug information about the current conversation context.",
        |cc| Box::pin(debug_command(cc)),
    );
    let cm_cmd = CommandTC::new("cm", "Changes the chat model.", |cc| {
        Box::pin(change_model_command(cc))
    });
    let help_cmd = CommandTC::new("help", "Displays this help message.", |cc| {
        Box::pin(help_command(cc))
    });
    let q_cmd = CommandTC::new(
        "quit",
        "Quits this program. Also 'q'.",
        |_cc| unreachable!(),
    );
    let gf_cmd = CommandTC::new(
        "gf",
        "Adds the contents of the specified files to the current context.",
        |cc| Box::pin(gf_command(cc)),
    );
    let readme_cmd = CommandTC::new(
        "readme",
        "Writes a readme for the given directory. Accepts optional file extensions.",
        |cc| Box::pin(readme_command(cc)),
    );
    let doc_cmd = CommandTC::new(
        "doc",
        "Generates documentation about the current chat context. Saves as Markdown.",
        |cc| Box::pin(document_command(cc)),
    );

    //register commands
    map.insert(debug_cmd.name, debug_cmd);
    map.insert(rmr_cmd.name, rmr_cmd);
    map.insert(clear_cmd.name, clear_cmd);
    map.insert(cm_cmd.name, cm_cmd);
    map.insert(help_cmd.name, help_cmd);
    map.insert(q_cmd.name, q_cmd);
    map.insert(gf_cmd.name, gf_cmd);
    map.insert(readme_cmd.name, readme_cmd);
    map.insert(doc_cmd.name, doc_cmd);

    //return commands
    map
});

#[derive(Clone)]
pub struct CommandContext {
    pub conversation_context: Arc<Mutex<ConversationContext>>,
    pub dev_message: Arc<Message>,
    pub chat_client: Arc<ChatClient>,
    pub cmd: String,
    pub args: Vec<String>,
}

impl CommandContext {
    pub fn new(
        conversation_context: Arc<Mutex<ConversationContext>>,
        dev_message: Arc<Message>,
        chat_client: Arc<ChatClient>,
        cmd: String,
        args: Vec<String>,
    ) -> Self {
        Self {
            conversation_context,
            dev_message,
            chat_client,
            cmd,
            args,
        }
    }
}

pub struct CommandTC {
    pub name: &'static str,
    pub description: &'static str,
    pub run: fn(CommandContext) -> Pin<Box<dyn Future<Output = CommandResult>>>,
}

impl CommandTC {
    pub fn new(
        name: &'static str,
        description: &'static str,
        run: fn(CommandContext) -> Pin<Box<dyn Future<Output = CommandResult>>>,
    ) -> Self {
        Self {
            name,
            description,
            run,
        }
    }
}

async fn clear_command(cc: CommandContext) -> CommandResult {
    {
        let mut ctx = cc.conversation_context.lock().await;
        ctx.input.clear();
        ctx.input.push((*cc.dev_message).clone());
    } // release lock

    Command::new("clear")
        .status()
        .expect("clear command failed");
    println!("\nConversation cleared.\n");
    Ok(())
}

async fn start_rmr(_cc: CommandContext) -> CommandResult {
    if !is_executable_installed("rmr") {
        eprintln!("rmr not found");
        return Ok(());
    }
    Command::new("rmr").status().expect("rmr failed");
    println!("\nleaving rmr...\n");
    println!("back to tc...\n");
    Ok(())
}

fn is_executable_installed(executable: &str) -> bool {
    if let Ok(paths) = env::var("PATH") {
        for path in env::split_paths(&paths) {
            let full_path = path.join(executable);
            if full_path.is_file() {
                if let Ok(metadata) = fs::metadata(&full_path) {
                    return !metadata.permissions().readonly();
                }
            }
        }
    }
    false
}

async fn debug_command(cc: CommandContext) -> CommandResult {
    let ctx = cc.conversation_context.lock().await;
    println!("\nCurrent model: {}", ctx.model);
    println!("\nCurrent context messages:\n");
    for msg in &ctx.input {
        println!("{}:\n{}\n:::\n", msg.role, msg.content);
    }
    println!();
    Ok(())
}

async fn change_model_command(cc: CommandContext) -> CommandResult {
    let mut ctx = cc.conversation_context.lock().await;
    const AVAILABLE_MODELS: &[&str] = &[
        "gpt-4o",
        "gpt-4-mini",
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
    Ok(())
}

async fn help_command(_cc: CommandContext) -> CommandResult {
    println!("\nAvailable commands:\n");
    for tc in TC_COMMANDS.values() {
        if tc.name.len() < 7 {
            println!("{:7} - {}", tc.name, tc.description);
        }
    }
    println!();
    Ok(())
}

async fn gf_command(cc: CommandContext) -> CommandResult {
    let mut ctx = cc.conversation_context.lock().await;
    if cc.args.is_empty() {
        eprintln!(
            "\nInvalid use of {}. Usage: {} <path1> <path2> ...\n",
            cc.cmd, cc.cmd
        );
        return Ok(());
    }

    for path in cc.args {
        let trimmed_path = path.trim();
        match fs::read_to_string(Path::new(trimmed_path)) {
            Ok(content) => {
                ctx.input.push(Message {
                    role: "user".to_string(),
                    content: format!("{}\n\n:::\n\n{}", trimmed_path, content),
                });
                println!("\nAdded {} to conversation context\n", trimmed_path);
            }
            Err(e) => eprintln!("Error reading {}: {}\n", trimmed_path, e),
        }
    }
    Ok(())
}

use std::collections::HashSet;
use walkdir::WalkDir;

async fn readme_command(cc: CommandContext) -> CommandResult {
    if cc.args.is_empty() {
        eprintln!("\nInvalid use of readme. Usage: readme <directory> [extensions...]\n");
        return Ok(());
    }
    let dir = cc.args[0].to_owned();

    if !Path::new(&dir).exists() {
        eprintln!("\nDirectory '{}' not found.\n", dir);
        return Ok(());
    }

    let extensions: HashSet<&str> = if cc.args.len() > 1 {
        cc.args.iter().skip(1).map(|s| s.as_str()).collect()
    } else {
        HashSet::new()
    };

    let mut new_context = ConversationContext::new("o3-mini", false);
    let dev_message = Message {
        role: "developer".into(),
        content: MESSAGES.get("readme").unwrap().to_string(),
    };
    new_context.input.push(dev_message);

    let mut names = vec![];
    for entry in WalkDir::new(dir) {
        let entry = entry?;

        if entry.file_type().is_file() && !entry.file_name().to_str().unwrap().starts_with('.') {
            let path = entry.path();
            if extensions.is_empty()
                || extensions.contains(path.extension().and_then(|ext| ext.to_str()).unwrap_or(""))
            {
                match fs::read_to_string(path) {
                    Ok(content) => {
                        names.push(path.display().to_string());
                        new_context.input.push(Message {
                            role: "user".to_string(),
                            content: format!("{}\n\n:::\n\n{}", path.display(), content),
                        });
                    }
                    Err(e) => eprintln!("Error reading {}: {}", path.display(), e),
                }
            }
        }
    }

    println!("\nFiles used: {:?}\n\n", names);
    let response = cc.chat_client.send_request(&new_context).await?;

    let result_content = if let Some(r) = extract_message_text(&response) {
        r
    } else {
        eprintln!("No content received from readme command.");
        return Ok(());
    };
    println!("{}", result_content);
    println!("\nEnter the README file name to save (without extension): ");

    let mut filename = String::new();
    std::io::stdin()
        .read_line(&mut filename)
        .expect("Failed to read input");
    let sanitized_filename = filename.trim().to_string();

    if sanitized_filename.is_empty() {
        eprintln!("Invalid filename. Document not saved.");
        return Ok(());
    }

    let final_name = format!("readmes/{}.md", sanitized_filename);

    println!(
        "\nDo you want to save this document as '{}.md'? (y/n): ",
        sanitized_filename
    );

    let mut answer = String::new();
    std::io::stdin()
        .read_line(&mut answer)
        .expect("Failed to read input");

    if answer.trim().eq_ignore_ascii_case("y") || answer.trim().eq_ignore_ascii_case("yes") {
        if !Path::new("readmes").exists() {
            fs::create_dir("readmes")?;
        }
        let mut file = File::create(&final_name)?;
        file.write_all(result_content.as_bytes())?;
        println!("\nDocument saved to '{}'\n", &final_name);
    } else {
        println!("Document not saved.\n");
    }
    Ok(())
}

fn extract_message_text(response: &Response) -> Option<String> {
    for output in &response.output {
        if output.type_field == "message" {
            if let Some(content) = &output.content {
                if let Some(first_content) = content.first() {
                    return Some(first_content.text.clone());
                }
            }
        }
    }
    None
}

async fn document_command(cc: CommandContext) -> CommandResult {
    let ctx = cc.conversation_context.lock().await;
    let mut new_context = ConversationContext::new("o3-mini", false);
    let dev_message = Message {
        role: "developer".into(),
        content: MESSAGES.get("document_prompt").unwrap().to_string(),
    };
    new_context.input.push(dev_message);
    for msg in &ctx.input {
        if msg.role != "developer" {
            new_context.input.push(msg.clone());
        }
    }

    let response = cc.chat_client.send_request(&new_context).await?;

    let report = if let Some(r) = extract_message_text(&response) {
        r
    } else {
        eprintln!("No content received in the document report.");
        return Ok(());
    };

    let mut title_context = ConversationContext::new("gpt-4o", false);
    let title_prompt = Message {
        role: "developer".into(),
        content: format!(
            "{} \n::\n {}",
            MESSAGES.get("title_prompt").unwrap(),
            report
        ),
    };
    title_context.input.push(title_prompt);
    let title_response = cc.chat_client.send_request(&title_context).await?;

    let title = if let Some(r) = extract_message_text(&title_response) {
        r
    } else {
        "Report".to_string()
    };

    let sanitized_title = title
        .replace("/", "_")
        .replace("\\", "_")
        .replace(" ", "_")
        .replace('"', "");

    let filename = format!("reports/{}.md", sanitized_title);
    let file_contents = format!("{}\n\n{}", title, report);
    println!("\n{}", file_contents);
    println!(
        "\nDo you want to save this document as '{}'? (y/n): ",
        filename
    );
    let mut answer = String::new();
    std::io::stdin()
        .read_line(&mut answer)
        .expect("Failed to read input");
    if answer.trim().eq_ignore_ascii_case("y") || answer.trim().eq_ignore_ascii_case("yes") {
        if !Path::new("reports").exists() {
            std::fs::create_dir("reports").unwrap();
        }
        let mut file = File::create(&filename).expect("Could not create file");
        file.write_all(file_contents.as_bytes())
            .expect("Could not write to file");
        println!("\nDocument saved as '{}'\n", filename);
    } else {
        println!("Document not saved.\n");
    }
    Ok(())
}
