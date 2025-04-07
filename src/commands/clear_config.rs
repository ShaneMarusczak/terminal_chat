use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use crate::tc_config::get_config_path;
use crate::utils::confirm_action;
use std::process::Command;

pub async fn dc(cc: Option<CommandContext>) -> CommandResult {
    if cc.is_some() {
        let path = get_config_path();
        let path_str = path.to_str().unwrap_or_default();

        if !path.exists() {
            println!("\nNo config file found at {}\n", path_str);
            return Ok(());
        }

        if confirm_action(&format!("Are you sure you want to delete {}?", path_str)) {
            Command::new("rm").arg(&path).status()?;
            println!("\nDeleted {}\n", path_str);
        } else {
            println!("\nDid not delete: {}\n", path_str);
        }
    }
    Ok(())
}
