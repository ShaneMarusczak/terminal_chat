use crate::{
    commands::command_context::CommandContext,
    commands::command_tc::CommandResult,
    conversation::Role,
    message_printer::print_message,
    tc_config::get_config,
};

pub async fn search_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
        let ctx = cc.conversation_context.lock().await;
        let config = get_config()?;

        if cc.args.is_empty() {
            print_message("Usage: :search <term>", Role::System, &config);
            return Ok(());
        }

        let search_term = cc.args.join(" ");
        let search_lower = search_term.to_lowercase();
        let mut found_count = 0;

        println!("\nSearching for: \"{}\"", search_term);
        println!("----------------------------------------");

        for (idx, message) in ctx.input.iter().enumerate() {
            if !message.role.is_visible() {
                continue;
            }

            if message.content.to_lowercase().contains(&search_lower) {
                found_count += 1;

                println!("\n[{}] {}:", idx, message.role.display_name());

                for line in message.content.lines() {
                    if line.to_lowercase().contains(&search_lower) {
                        let trimmed = line.trim();
                        if trimmed.len() > 100 {
                            if let Some(pos) = trimmed.to_lowercase().find(&search_lower) {
                                let start = pos.saturating_sub(40);
                                let end =
                                    (pos + search_lower.len() + 40).min(trimmed.len());
                                let snippet = &trimmed[start..end];
                                println!("  ...{}...", snippet);
                            }
                        } else {
                            println!("  {}", trimmed);
                        }
                    }
                }
            }
        }

        println!("\n----------------------------------------");
        let summary = format!("Found {} match(es)", found_count);
        print_message(&summary, Role::System, &config);
    }
    Ok(())
}
