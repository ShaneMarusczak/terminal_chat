use crate::{
    commands::command_context::CommandContext,
    commands::command_tc::CommandResult,
    message_printer::{MessageType, print_message},
    tc_config::get_config,
};

pub async fn show_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
        let mut ctx = cc.conversation_context.lock().await;
        let config = get_config()?;

        if cc.args.is_empty() {
            print_message("Usage: :show <message_number>", MessageType::System, &config);
            return Ok(());
        }

        let message_num: usize = cc.args[0].parse()
            .map_err(|_| "Invalid message number")?;

        if message_num >= ctx.input.len() {
            let error_msg = format!("Message {} not found. Valid range: 0-{}",
                message_num, ctx.input.len() - 1);
            print_message(&error_msg, MessageType::System, &config);
            return Ok(());
        }

        let message = &ctx.input[message_num];

        // Skip developer messages
        if message.role == "developer" {
            print_message("Cannot display developer/system messages", MessageType::System, &config);
            return Ok(());
        }

        let role_display = match message.role.as_str() {
            "user" => "You",
            "assistant" => "AI",
            _ => &message.role,
        };

        println!("\n========================================");
        println!("[{}] {}", message_num, role_display);
        println!("========================================\n");

        print_message(&message.content,
            if message.role == "user" { MessageType::User } else { MessageType::Assistant },
            &config);

        println!("\n========================================");

        // Store in yank_target so it can be yanked
        ctx.yank_target = Some(message.content.clone());

        print_message("Message loaded (use :y to copy)", MessageType::System, &config);
    }
    Ok(())
}
