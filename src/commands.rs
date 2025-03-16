use crate::{
    chat_client::ChatClient,
    conversation::{ConversationContext, Message},
    messages::MESSAGES,
};
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{Write, stdin},
    path::Path,
    process::Command,
};
const AVAILABLE_MODELS: &[&str] = &["gpt-4o", "gpt-4o-search-preview", "o1", "o3-mini"];

pub async fn handle_command(
    cmd: &str,
    context: &mut ConversationContext,
    dev_message: &Message,
    chat_client: &ChatClient,
) -> Result<(), Box<dyn Error>> {
    let cmd = cmd.trim();
    match cmd {
        "clear" => clear_command(context, dev_message),
        "debug" => debug_command(context),
        "doc" => document_command(context, chat_client).await?,
        "cm" => change_model_command(context),
        "help" => help_command(),
        "rmr" => start_rmr(),
        _ => {
            if cmd.starts_with("gf") {
                gf_command(context, cmd);
            } else {
                eprintln!("Unknown command: {cmd}");
            }
        }
    }
    Ok(())
}

fn start_rmr() {
    if !is_executable_installed("rmr") {
        eprintln!("rmr not found");
        return;
    }
    Command::new("rmr").status().expect("rmr failed");
    println!("\nleaving rmr...\n");
    println!("back to tc...\n");
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

fn clear_command(context: &mut ConversationContext, dev_message: &Message) {
    context.messages.clear();
    context.messages.push(dev_message.clone());
    Command::new("clear")
        .status()
        .expect("clear command failed");
    println!("\nConversation cleared.\n");
}

fn debug_command(context: &ConversationContext) {
    println!("\nCurrent model: {}", context.model);
    println!("\nCurrent context messages:\n");
    for msg in &context.messages {
        println!("{}: {}\n", msg.role, msg.content);
    }
    println!();
}

fn change_model_command(context: &mut ConversationContext) {
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
            context.model = AVAILABLE_MODELS[num - 1].to_string();
            println!("Model changed to: {}\n", context.model);
        }
        _ => eprintln!(
            "Invalid selecton. Keeping current model: {}\n",
            context.model
        ),
    }
}

fn gf_command(context: &mut ConversationContext, cmd: &str) {
    let args = cmd.trim().splitn(2, ' ').collect::<Vec<&str>>();
    if args.len() == 2 {
        let path = args[1].trim();
        match fs::read_to_string(Path::new(path)) {
            Ok(content) => {
                context.messages.push(Message {
                    role: "user".to_string(),
                    content: format!("{}\n\n:::\n\n{}", path, content),
                });
                println!("\nAdded {} to conversation context\n", path);
            }
            Err(e) => eprintln!("Error reading {}:{}\n", path, e),
        }
    } else {
        eprintln!("Invalid use of gf. Usage: gf <path>");
    }
}

fn help_command() {
    println!("\nAvailable commands:");
    println!("clear      - Clears the conversation context.");
    println!("debug      - Prints debugging information for the current conversation.");
    println!("doc        - Documents the conversation using the chat client's document method.");
    println!("cm         - Changes the chat model.");
    println!("help       - Displays this help message.");
    println!("gf <path>  - Adds the content of the specified file to the conversation context.");
    println!("rmr        - Launches rmr if installed in this machines path");

    println!();
}

async fn document_command(
    context: &ConversationContext,
    chat_client: &ChatClient,
) -> Result<(), Box<dyn Error>> {
    //build context
    let mut new_context = ConversationContext::new("o3-mini");
    let dev_message = Message {
        role: "developer".into(),
        content: MESSAGES.get("document_prompt").unwrap().to_string(),
    };
    new_context.messages.push(dev_message);
    for msg in &context.messages {
        if msg.role != "developer" {
            new_context.messages.push(msg.clone());
        }
    }

    //get report
    let response = chat_client.send_request(&new_context).await?;
    let report = if let Some(choice) = response.choices.first() {
        choice.message.content.clone()
    } else {
        eprintln!("No report content received.");
        return Ok(());
    };

    //get title
    let mut title_context = ConversationContext::new("gpt-4o");
    let title_prompt = Message {
        role: "developer".into(),
        content: format!(
            "{} \n::\n {}",
            MESSAGES.get("title_prompt").unwrap(),
            report
        ),
    };
    title_context.messages.push(title_prompt);
    let title_response = chat_client.send_request(&title_context).await?;
    let title = if let Some(choice) = title_response.choices.first() {
        choice.message.content.trim().to_string()
    } else {
        "Report".to_string()
    };
    let sanitized_title = title
        .replace("/", "_")
        .replace("\\", "_")
        .replace(" ", "_")
        .replace('"', "");

    //save to file system
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
        println!("Document not saved.");
    }
    Ok(())
}
