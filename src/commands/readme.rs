use crate::commands::command_context::CommandContext;
use crate::conversation::{ConversationContext, Message};
use crate::messages::MESSAGES;
use crate::utils::{confirm_action, extract_message_text, read_user_input};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;

use super::CommandResult;

pub async fn readme_command(cc: CommandContext) -> CommandResult {
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
        content: MESSAGES.get("readme").unwrap_or(&"").to_string(),
    };
    new_context.input.push(dev_message);

    let mut names = vec![];
    for entry in WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_type().is_file()
            && !entry.file_name().to_str().unwrap_or(".").starts_with('.')
        {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if extensions.is_empty() || extensions.contains(ext) {
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
    let sanitized_filename =
        read_user_input("\nEnter the README file name to save (without extension): ");

    if sanitized_filename.is_empty() {
        eprintln!("Invalid filename. Document not saved.");
        return Ok(());
    }

    let final_name = format!("readmes/{}.md", sanitized_filename);
    println!(
        "\nDo you want to save this document as '{}.md'? (y/n): ",
        sanitized_filename
    );

    if confirm_action(&format!(
        "\nDo you want to save this document as '{}.md'? (y/n): ",
        sanitized_filename
    )) {
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
