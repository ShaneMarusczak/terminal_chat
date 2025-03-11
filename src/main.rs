use std::{
    io::{Write, stdout},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use rustyline::error::ReadlineError;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    println!("\n  -- terminal chat -- \n");

    let mut conversation_context = ConversationContext::new();
    let dev_message = Message {
        role: "developer".into(),
        content: "You are helpful, intelligent, and friendly.
You are also very concise and accurate.
No words are wasted in your responses.
Always answer with very accurate and kind responses that are short, to the point and friendly."
            .to_owned(),
    };
    conversation_context.messages.push(dev_message.clone());
    let client = reqwest::Client::new();
    let url = "https://api.openai.com/v1/chat/completions";
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

    let mut rl = rustyline::DefaultEditor::new().unwrap();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let line = line.trim().to_owned();
                if line.is_empty() {
                    continue;
                }
                rl.add_history_entry(&line).unwrap();
                if line.starts_with(':') {
                    let command = line.trim_start_matches(':');
                    match command {
                        "q" => break,
                        "clear" => {
                            conversation_context.messages.clear();
                            conversation_context.messages.push(dev_message.clone());
                            println!("\nConversation cleared.\n");
                        }
                        "debug" => {
                            println!("\nDebugging conversation messages:");
                            for message in &conversation_context.messages {
                                println!("{}: {}", message.role, message.content);
                            }
                            println!();
                        }
                        _ => eprintln!("\nInvalid command: {}\n", command),
                    }
                } else {
                    conversation_context.messages.push(Message {
                        role: "user".into(),
                        content: line.clone(),
                    });
                    let request_as_json = serde_json::to_string(&conversation_context).unwrap();
                    let response = run_with_spinner(async {
                        client
                            .post(url)
                            .header("Content-Type", "application/json")
                            .header("Authorization", format!("Bearer {}", api_key))
                            .body(request_as_json)
                            .send()
                            .await?
                            .text()
                            .await
                    })
                    .await?;
                    print!("\r                   \r");
                    let _ = stdout().flush();
                    let response_data: Response = serde_json::from_str(&response).unwrap();
                    if let Some(choice) = response_data.choices.first() {
                        let content = &choice.message.content;
                        conversation_context.messages.push(Message {
                            role: "assistant".into(),
                            content: content.to_owned(),
                        });
                        println!();
                        println!("{}", content);
                        println!();
                    }
                }
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}

async fn run_with_spinner<F, T>(f: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let (spinner_running, spinner_handle) = start_spinner();
    let result = f.await;
    spinner_running.store(false, Ordering::Relaxed);
    let _ = spinner_handle.await;
    result
}

fn start_spinner() -> (Arc<AtomicBool>, tokio::task::JoinHandle<()>) {
    let spinner_running = Arc::new(AtomicBool::new(true));
    let spinner_flag = spinner_running.clone();
    let handle = tokio::spawn(async move {
        let spinner_states = [
            " └[-   ]┐   ",
            "  ┌[ -  ]┘  ",
            "   └[  - ]┐ ",
            "    ┌[   -]┘",
            "   └[  - ]┐ ",
            "  ┌[ -  ]┘  ",
            " └[-   ]┐   ",
        ];
        let mut i = 0;
        while spinner_flag.load(Ordering::Relaxed) {
            print!(
                "\r\x1b[36m{}\x1b[0m",
                spinner_states[i % spinner_states.len()]
            );
            let _ = stdout().flush();
            i += 1;
            sleep(Duration::from_millis(125)).await;
        }
    });
    (spinner_running, handle)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Debug)]
struct ConversationContext {
    model: String,
    messages: Vec<Message>,
}

impl ConversationContext {
    fn new() -> Self {
        ConversationContext {
            model: "gpt-4o".into(),
            messages: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Choice {
    message: Message,
}
