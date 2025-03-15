mod chat_client;
mod commands;
mod conversation;
mod spinner;

use chat_client::ChatClient;
use commands::handle_command;
use conversation::{ConversationContext, Message};
use rustyline::error::ReadlineError;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n  -- terminal chat -- \n");
    let mut conversation_context = ConversationContext::new("gpt-4o");
    let developer_message = Message {
        role: "developer".into(),
        content: "You are helpful, intelligent, and friendly.
You are also very concise and accurate.
No words are wasted in your responses.
When what is being asked is ambiguous, please ask clarifying questions before answering.
Always answer with very accurate and kind responses that are short, to the point and friendly."
            .into(),
    };
    conversation_context
        .messages
        .push(developer_message.clone());
    let chat_client = ChatClient::new()?;
    let mut rl = rustyline::DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline(">> ");
        let line = match readline {
            Ok(l) => l.trim().to_owned(),
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        };
        if line.is_empty() {
            continue;
        }
        rl.add_history_entry(&line).unwrap();
        if handle_command(
            &line,
            &mut conversation_context,
            &developer_message,
            &chat_client,
        )
        .await?
        {
            break;
        }
    }
    Ok(())
}
