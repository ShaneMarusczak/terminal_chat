use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use crate::utils::read_user_input;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub async fn sc_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
        let current_convo = cc.conversation_context.lock().await;

        let as_json = serde_json::to_string(&*current_convo)?;

        let convo_name = read_user_input("Conversation name: ");

        if !Path::exists(Path::new("conversations")) {
            fs::create_dir("conversations")?;
        }

        let path = format!("conversations/{convo_name}.json");

        let mut file = File::create(&path).map_err(|_| "Could not create file")?;
        file.write_all(as_json.as_bytes())
            .map_err(|_| "Could not write to file")?;
    }
    Ok(())
}
