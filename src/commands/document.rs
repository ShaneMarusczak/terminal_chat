use crate::commands::command_context::CommandContext;
use crate::conversation::{ConversationContext, Message};
use crate::messages::MESSAGES;
use crate::utils::{confirm_action, extract_message_text};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use super::CommandResult;

pub async fn document_command(cc: CommandContext) -> CommandResult {
    let ctx = cc.conversation_context.lock().await;
    let mut new_context = ConversationContext::new("o3-mini", false);

    let dev_message = Message {
        role: "developer".into(),
        content: MESSAGES
            .get("document_prompt")
            .ok_or("Missing document_prompt")?
            .to_string(),
    };
    new_context.input.push(dev_message);

    for msg in &ctx.input {
        if msg.role != "developer" {
            new_context.input.push(msg.clone());
        }
    }

    let response = cc.chat_client.send_request("d", &new_context).await?;
    let report =
        extract_message_text(&response).ok_or("No content received in the document report")?;

    let mut title_context = ConversationContext::new("gpt-4o", false);
    let title_prompt = Message {
        role: "developer".into(),
        content: format!(
            "{} \n::\n {}",
            MESSAGES.get("title_prompt").ok_or("Missing title_prompt")?,
            report
        ),
    };
    title_context.input.push(title_prompt);

    let title_response = cc.chat_client.send_request("d", &title_context).await?;
    let title = extract_message_text(&title_response).unwrap_or_else(|| "Report".to_string());

    let sanitized_title = title
        .replace("/", "_")
        .replace("\\", "_")
        .replace(" ", "_")
        .replace('"', "");

    let filename = format!("reports/{}.md", sanitized_title);
    let file_contents = format!("{}\n\n{}", title, report);
    println!("\n{}", file_contents);

    if confirm_action(&format!(
        "\nDo you want to save this document as '{}'? (y/n): ",
        filename
    )) {
        if !Path::new("reports").exists() {
            fs::create_dir("reports").map_err(|_| "Could not create reports directory")?;
        }

        let mut file = File::create(&filename).map_err(|_| "Could not create file")?;
        file.write_all(file_contents.as_bytes())
            .map_err(|_| "Could not write to file")?;
        println!("\nDocument saved as '{}'\n", filename);
    } else {
        println!("Document not saved.\n");
    }

    Ok(())
}
