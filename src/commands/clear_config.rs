use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use crate::tc_config::get_config_path;
use crate::utils::confirm_action;
use std::process::Command;

pub async fn dc(cc: Option<CommandContext<'_>>) -> CommandResult {
    if cc.is_some() {
        let path = get_config_path();

        if !path.exists() {
            println!("\nNo config file found at {}\n", path.to_str().unwrap());
            return Ok(());
        }

        if confirm_action(&format!(
            "Are you sure you want to delete {}?",
            path.to_str().unwrap()
        )) {
            Command::new("rm").arg(&path).status()?;
            println!("\nDeleted {}\n", path.to_str().unwrap());
        } else {
            println!("\nDid not delete: {}\n", path.to_str().unwrap());
        }
    }
    Ok(())
}
