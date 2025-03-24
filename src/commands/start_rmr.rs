use crate::commands::command_context::CommandContext;
use std::env;
use std::fs;
use std::process::Command;

use super::CommandResult;

pub async fn start_rmr(_cc: CommandContext) -> CommandResult {
    if !is_executable_installed("rmr") {
        eprintln!("rmr not found");
        return Ok(());
    }
    Command::new("rmr").status().expect("rmr failed");
    println!("\nleaving rmr...\n");
    println!("back to tc...\n");
    Ok(())
}

fn is_executable_installed(executable: &str) -> bool {
    if let Ok(paths) = env::var("PATH") {
        for path in env::split_paths(&paths) {
            let full_path = path.join(executable);
            if full_path.is_file() {
                if let Ok(metadata) = fs::metadata(&full_path) {
                    return !metadata.permissions().readonly();
                }
            }
        }
    }
    false
}
