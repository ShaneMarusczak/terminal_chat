use crate::{
    chat_client::ChatClient,
    conversation::{ConversationContext, Message, Response},
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

const AVAILABLE_MODELS: &[&str] = &[
    "gpt-4o",
    "gpt-4o-mini",
    "gpt-4o-search-preview",
    "o1",
    "o3-mini",
];

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
            } else if cmd.starts_with("readme") {
                readme_command(chat_client, cmd).await?;
            } else {
                let main_cmd = cmd.split_whitespace().next().unwrap_or(cmd);
                let commands = [
                    "clear", "debug", "doc", "cm", "help", "rmr", "gf", "readme", "q", "quit",
                ];
                eprintln!("\nUnknown command: {main_cmd}");

                let maybe = find_matching_word(main_cmd, &commands)?;
                eprint!("Did you mean {}?\n\n", maybe);
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
    context.input.clear();
    context.input.push(dev_message.clone());
    Command::new("clear")
        .status()
        .expect("clear command failed");
    println!("\nConversation cleared.\n");
}

fn debug_command(context: &ConversationContext) {
    println!("\nCurrent model: {}", context.model);
    println!("\nCurrent context messages:\n");
    for msg in &context.input {
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
            "Invalid selection. Keeping current model: {}\n",
            context.model
        ),
    }
}

fn gf_command(context: &mut ConversationContext, cmd: &str) {
    let args = cmd.split_whitespace().skip(1).collect::<Vec<&str>>();
    if args.is_empty() {
        eprintln!("Invalid use of gf. Usage: gf <path1> <path2> ...");
        return;
    }

    for path in args {
        let trimmed_path = path.trim();
        match fs::read_to_string(Path::new(trimmed_path)) {
            Ok(content) => {
                context.input.push(Message {
                    role: "user".to_string(),
                    content: format!("{}\n\n:::\n\n{}", trimmed_path, content),
                });
                println!("\nAdded {} to conversation context\n", trimmed_path);
            }
            Err(e) => eprintln!("Error reading {}: {}\n", trimmed_path, e),
        }
    }
}

fn help_command() {
    println!("\nAvailable commands:");
    println!("clear      - Clears the conversation context.");
    println!("debug      - Prints debugging information for the current conversation.");
    println!("doc        - Documents the conversation using the chat client's document method.");
    println!("cm         - Changes the chat model.");
    println!("rmr        - Launches rmr if installed in this machine's path.");
    println!("help       - Displays this help message.");
    println!("gf <path1> <path2> ...");
    println!("           - Adds the content of specified files to the conversation context.");
    println!("readme <directory> [extensions...]");
    println!("           - Processes directory files into a README document.");
    println!();
}

async fn document_command(
    context: &ConversationContext,
    chat_client: &ChatClient,
) -> Result<(), Box<dyn Error>> {
    let mut new_context = ConversationContext::new("o3-mini", false);
    let dev_message = Message {
        role: "developer".into(),
        content: MESSAGES.get("document_prompt").unwrap().to_string(),
    };
    new_context.input.push(dev_message);
    for msg in &context.input {
        if msg.role != "developer" {
            new_context.input.push(msg.clone());
        }
    }

    let response = chat_client.send_request(&new_context).await?;

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
    let title_response = chat_client.send_request(&title_context).await?;

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

use std::collections::HashSet;
use walkdir::WalkDir;

async fn readme_command(chat_client: &ChatClient, cmd: &str) -> Result<(), Box<dyn Error>> {
    let args = cmd.split_whitespace().skip(1).collect::<Vec<&str>>();
    if args.is_empty() {
        eprintln!("Invalid use of readme. Usage: readme <directory> [extensions...]");
        return Ok(());
    }
    let dir = args[0];

    if !Path::new(dir).exists() {
        eprintln!("\nDirectory '{}' not found.\n", dir);
        return Ok(());
    }

    let extensions: HashSet<&str> = if args.len() > 1 {
        args[1..].iter().copied().collect()
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
            let file_name = String::from(path.to_str().unwrap());
            if extensions.is_empty()
                || extensions.contains(path.extension().and_then(|ext| ext.to_str()).unwrap_or(""))
            {
                match fs::read_to_string(path) {
                    Ok(content) => {
                        names.push(file_name);
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
    let response = chat_client.send_request(&new_context).await?;

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
        println!("Document not saved.");
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

fn find_matching_word(word: &str, words: &[&str]) -> Result<String, String> {
    let mut min_dist = 9999;
    let mut final_string = String::new();
    for w in words {
        let distance = min_distance(w, word);
        if distance < min_dist {
            min_dist = distance;
            final_string = String::from(*w);
        }
    }
    Ok(final_string)
}

fn min_distance(word1: &str, word2: &str) -> i32 {
    let (word1, word2) = (word1.as_bytes(), word2.as_bytes());
    let mut dist = Vec::with_capacity(word2.len() + 1);
    for j in 0..=word2.len() {
        dist.push(j)
    }
    let mut prev_dist = dist.clone();
    for i in 1..=word1.len() {
        for j in 0..=word2.len() {
            if j == 0 {
                dist[j] += 1;
            } else if word1[i - 1] == word2[j - 1] {
                dist[j] = prev_dist[j - 1];
            } else {
                dist[j] = dist[j].min(dist[j - 1]).min(prev_dist[j - 1]) + 1;
            }
        }
        prev_dist.copy_from_slice(&dist);
    }
    dist[word2.len()] as i32
}
