use crate::chat_client::chat;
use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use crate::conversation::{ConversationContext, Message, Role};
use crate::messages::MESSAGES;
use crate::tc_config::{ConfigTC, get_config};
use crate::utils::{confirm_action, read_user_input, select_model, walk_directory};
use std::collections::HashSet;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Generate a README from directory contents. Returns the generated text.
/// No file I/O for output, no interactive prompts.
pub async fn generate_readme(
    dir: &str,
    extensions: &HashSet<&str>,
    model: &str,
    config: &ConfigTC,
) -> Result<String, Box<dyn Error>> {
    if !Path::new(dir).exists() {
        return Err(format!("Directory '{}' not found.", dir).into());
    }

    let mut ctx = ConversationContext::new(model);
    ctx.input.push(Message {
        role: Role::Developer,
        content: MESSAGES
            .get("readme")
            .ok_or("Missing readme prompt")?
            .to_string(),
    });

    let excluded_dirs = HashSet::from(["target"]);
    for (path, content) in walk_directory(dir, extensions, &excluded_dirs)? {
        ctx.input.push(Message {
            role: Role::User,
            content: format!("{}\n\n:::\n\n{}", path, content),
        });
    }

    let result = chat(&ctx, config, None).await?;
    // Models occasionally emit U+2022 (•) for bullets; normalize to ASCII
    // dashes so the output is plain markdown.
    Ok(result.replace('•', "-"))
}

pub async fn readme_command(cc: Option<CommandContext>) -> CommandResult {
    let Some(cc) = cc else { return Ok(()) };

    if cc.args.is_empty() {
        eprintln!("\nInvalid use of readme. Usage: readme <directory> [extensions...]\n");
        return Ok(());
    }
    let dir = cc.args[0].clone();
    let extensions: HashSet<&str> = cc.args.iter().skip(1).map(String::as_str).collect();

    let config = get_config()?;
    let selected_model = select_model(&config.all_models, "Select a model for README generation:")?;

    let result_content = match generate_readme(&dir, &extensions, &selected_model, &config).await {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error generating README: {}", e);
            return Ok(());
        }
    };

    println!("\n{}\n", result_content);
    let sanitized_filename =
        read_user_input("\nEnter the README file name to save (without extension): ")?;

    if sanitized_filename.is_empty() {
        eprintln!("Invalid filename. Document not saved.");
        return Ok(());
    }

    let final_name = format!("readmes/{}.md", sanitized_filename);

    if !confirm_action(&format!(
        "\nDo you want to save this document as '{}.md'? (y/n): ",
        sanitized_filename
    )) {
        println!("Document not saved.\n");
        return Ok(());
    }

    fs::create_dir_all("readmes")
        .map_err(|e| format!("Could not create readmes directory: {}", e))?;
    let mut file = File::create(&final_name)
        .map_err(|e| format!("Could not create file '{}': {}", final_name, e))?;
    file.write_all(result_content.as_bytes())
        .map_err(|e| format!("Could not write to file '{}': {}", final_name, e))?;
    println!("\nDocument saved to '{}'\n", &final_name);

    Ok(())
}
