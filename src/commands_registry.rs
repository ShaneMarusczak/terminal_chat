use tokio::sync::Mutex;

use crate::{
    chat_client::ChatClient,
    conversation::{ConversationContext, Message},
};
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs,
    future::Future,
    pin::Pin,
    process::Command,
    sync::{Arc, LazyLock},
};

pub static TC_COMMANDS: LazyLock<HashMap<&str, CommandTC>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    //define commands
    let clear_cmd = CommandTC::new("clear", "Clears the conversation context.", |cc| {
        Box::pin(clear_command(cc))
    });
    let rmr_cmd = CommandTC::new("rmr", "Launches rmr if isntalled.", |cc| {
        Box::pin(start_rmr(cc))
    });
    let debug_cmd = CommandTC::new(
        "debug",
        "Prints debug information about the current conversation context.",
        |cc| Box::pin(debug_command(cc)),
    );

    //register commands
    map.insert(debug_cmd.name, debug_cmd);
    map.insert(rmr_cmd.name, rmr_cmd);
    map.insert(clear_cmd.name, clear_cmd);

    //return commands
    map
});

#[derive(Clone)]
pub struct CommandContext {
    pub conversation_context: Arc<Mutex<ConversationContext>>,
    pub dev_message: Arc<Message>,
    pub chat_client: Arc<ChatClient>,
    pub cmd: String,
    pub args: Vec<String>,
}

impl CommandContext {
    pub fn new(
        conversation_context: Arc<Mutex<ConversationContext>>,
        dev_message: Arc<Message>,
        chat_client: Arc<ChatClient>,
        cmd: String,
        args: Vec<String>,
    ) -> Self {
        Self {
            conversation_context,
            dev_message,
            chat_client,
            cmd,
            args,
        }
    }
}

pub struct CommandTC {
    pub name: &'static str,
    pub description: &'static str,
    pub run: fn(CommandContext) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>>>>,
}

impl CommandTC {
    pub fn new(
        name: &'static str,
        description: &'static str,
        run: fn(CommandContext) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>>>>,
    ) -> Self {
        Self {
            name,
            description,
            run,
        }
    }
}

async fn clear_command(cc: CommandContext) -> Result<(), Box<dyn Error>> {
    let mut ctx = cc.conversation_context.lock().await;
    ctx.input.clear();
    ctx.input.push((*cc.dev_message).clone());
    drop(ctx); // Release the lock before running an external command

    Command::new("clear")
        .status()
        .expect("clear command failed");
    println!("\nConversation cleared.\n");
    Ok(())
}

async fn start_rmr(_cc: CommandContext) -> Result<(), Box<dyn Error>> {
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

async fn debug_command(cc: CommandContext) -> Result<(), Box<dyn Error>> {
    let ctx = cc.conversation_context.lock().await;
    println!("\nCurrent model: {}", ctx.model);
    println!("\nCurrent context messages:\n");
    for msg in &ctx.input {
        println!("{}:\n{}\n:::\n", msg.role, msg.content);
    }
    println!();
    Ok(())
}
