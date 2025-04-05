use crate::chat_client::send_request;
use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use crate::conversation::{ConversationContext, Message};
use crate::messages::MESSAGES;
use crate::preview_md::preview_markdown;
use crate::utils::{confirm_action, extract_message_text, read_user_input, walk_directory};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub async fn readme_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
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

        let mut excluded_dirs = HashSet::new();
        excluded_dirs.insert("target");

        let mut names = vec![];
        let files = walk_directory(&dir, &extensions, &excluded_dirs)?;

        for (path, content) in files {
            names.push(path.clone());
            new_context.input.push(Message {
                role: "user".to_string(),
                content: format!("{}\n\n:::\n\n{}", path, content),
            });
        }
        println!("\nFiles used: {:?}\n\n", names);
        let response = send_request("d", &new_context).await?;

        let result_content = if let Some(r) = extract_message_text(&response) {
            r.replace("•", "-")
        } else {
            eprintln!("No content received from readme command.");
            return Ok(());
        };

        preview_markdown(&result_content);
        let sanitized_filename =
            read_user_input("\nEnter the README file name to save (without extension): ");

        if sanitized_filename.is_empty() {
            eprintln!("Invalid filename. Document not saved.");
            return Ok(());
        }

        let final_name = format!("readmes/{}.md", sanitized_filename);

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
    }
    Ok(())
}
