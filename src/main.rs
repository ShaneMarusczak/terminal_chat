use std::error::Error;

// Import from the library
use tc::*;

// Only declare the run module which is binary-specific
mod run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        1 => run::as_repl().await?,
        _ => run::as_cli_tool(&args[1..]).await?,
    }

    Ok(())
}
