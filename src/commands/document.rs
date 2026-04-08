use crate::chat_client::chat;
use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use crate::conversation::{ConversationContext, Message, Role};
use crate::messages::MESSAGES;
use crate::tc_config::{ConfigTC, get_config};
use crate::utils::{confirm_action, select_model};
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Generate a document from a slice of messages. Returns `(title, report)`.
/// Filters out Developer-role messages, runs the document prompt, then runs
/// the title prompt. No file I/O or interactive prompts.
pub async fn generate_document(
    messages: &[Message],
    model: &str,
    config: &ConfigTC,
) -> Result<(String, String), Box<dyn Error>> {
    let report = run_prompt(
        model,
        config,
        prompt_message("document_prompt")?,
        messages.iter().filter(|m| m.role != Role::Developer).cloned(),
    )
    .await?;

    // Title generation is best-effort: a failed title call falls back to
    // "Report" so the document still saves.
    let title_seed = format!("{} \n::\n {}", prompt_message("title_prompt")?, report);
    let title = run_prompt(model, config, title_seed, std::iter::empty())
        .await
        .unwrap_or_else(|_| "Report".to_string());

    Ok((title, report))
}

fn prompt_message(key: &'static str) -> Result<String, Box<dyn Error>> {
    MESSAGES
        .get(key)
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Missing {}", key).into())
}

async fn run_prompt(
    model: &str,
    config: &ConfigTC,
    system: String,
    user_messages: impl IntoIterator<Item = Message>,
) -> Result<String, Box<dyn Error>> {
    let mut ctx = ConversationContext::new(model);
    ctx.input.push(Message {
        role: Role::Developer,
        content: system,
    });
    ctx.input.extend(user_messages);
    chat(&ctx, config, None).await
}

fn sanitize_filename(title: &str) -> String {
    title
        .replace(['/', '\\', ' '], "_")
        .replace('"', "")
}

pub async fn document_command(cc: Option<CommandContext>) -> CommandResult {
    let Some(cc) = cc else { return Ok(()) };

    let config = get_config()?;
    let selected_model = select_model(
        &config.all_models,
        "Select a model for document generation:",
    )?;

    let messages = {
        let ctx = cc.conversation_context.lock().await;
        ctx.input.clone()
    };

    let (title, report) = generate_document(&messages, &selected_model, &config).await?;
    let filename = format!("reports/{}.md", sanitize_filename(&title));
    let file_contents = format!("{}\n\n{}", title, report);

    println!("\n{}\n", file_contents);

    if !confirm_action(&format!(
        "\nDo you want to save this document as '{}'? (y/n): ",
        filename
    )) {
        println!("Document not saved.\n");
        return Ok(());
    }

    if !Path::new("reports").exists() {
        fs::create_dir("reports")
            .map_err(|e| format!("Could not create reports directory: {}", e))?;
    }

    let mut file = File::create(&filename)
        .map_err(|e| format!("Could not create file '{}': {}", filename, e))?;
    file.write_all(file_contents.as_bytes())
        .map_err(|e| format!("Could not write to file '{}': {}", filename, e))?;
    println!("\nDocument saved as '{}'\n", filename);

    Ok(())
}
