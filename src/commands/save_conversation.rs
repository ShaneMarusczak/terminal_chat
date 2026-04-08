use crate::commands::command_context::CommandContext;
use crate::commands::command_tc::CommandResult;
use crate::utils::read_user_input;
use std::fs;
use std::io::Write;

pub async fn sc_command(cc: Option<CommandContext>) -> CommandResult {
    if let Some(cc) = cc {
        let current_convo = cc.conversation_context.lock().await;

        let as_json = serde_json::to_string_pretty(&*current_convo)?;
        let convo_name = read_user_input("Conversation name: ")?;

        let mut path = dirs::home_dir().ok_or("Could not find home directory")?;
        path.push(".tc");
        path.push("conversations");
        fs::create_dir_all(&path)?;
        path.push(format!("{convo_name}.json"));

        let mut file = fs::File::create(&path)
            .map_err(|e| format!("Could not create file '{}': {}", path.display(), e))?;
        file.write_all(as_json.as_bytes())
            .map_err(|e| format!("Could not write to file '{}': {}", path.display(), e))?;

        println!("Conversation saved to: {}", path.display());
    }
    Ok(())
}
