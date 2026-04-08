use std::error::Error;
use std::io::IsTerminal;

use tc::*;

mod run;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let has_args = args.len() > 1;
    let stdin_is_tty = std::io::stdin().is_terminal();

    if !has_args && stdin_is_tty {
        run::as_repl().await?;
    } else {
        run::as_cli_tool(&args[1..]).await?;
    }

    Ok(())
}
