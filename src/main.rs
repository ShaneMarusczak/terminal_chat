use std::error::Error;

mod chat_client;
mod commands;
mod conversation;
mod message_printer;
mod messages;
mod preview_md;
mod run;
mod spinner;
mod tc_config;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        1 => run::as_repl().await?,
        _ => run::as_cli_tool(&args[1..]).await?,
    }

    Ok(())
}
