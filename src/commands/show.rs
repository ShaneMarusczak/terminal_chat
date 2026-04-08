use crate::{
    commands::command_context::CommandContext,
    commands::command_tc::CommandResult,
    conversation::Role,
    message_printer::print_message,
    tc_config::get_config,
};

pub async fn show_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
        let ctx = cc.conversation_context.lock().await;
        let config = get_config()?;

        if cc.args.is_empty() {
            print_message("Usage: :show <message_number>", Role::System, &config);
            return Ok(());
        }

        let message_num: usize = cc.args[0]
            .parse()
            .map_err(|_| "Invalid message number")?;

        if message_num >= ctx.input.len() {
            let error_msg = format!(
                "Message {} not found. Valid range: 0-{}",
                message_num,
                ctx.input.len() - 1
            );
            print_message(&error_msg, Role::System, &config);
            return Ok(());
        }

        let message = &ctx.input[message_num];

        if !message.role.is_visible() {
            print_message("Cannot display system messages", Role::System, &config);
            return Ok(());
        }

        println!("\n[{}] {}:", message_num, message.role.display_name());
        println!("----------------------------------------");
        print_message(&message.content, message.role, &config);
        println!("----------------------------------------");
    }
    Ok(())
}
