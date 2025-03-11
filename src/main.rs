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

#[derive(Debug)]
enum Command {
    Quit,
    Clear,
    Debug,
    Doc,
    Unknown(String),
}

fn parse_command(input: &str) -> Command {
    match input.trim_start_matches(':').trim() {
        "q" => Command::Quit,
        "clear" => Command::Clear,
        "debug" => Command::Debug,
        "doc" => Command::Doc,
        other => Command::Unknown(other.to_string()),
    }
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    println!("\n  -- terminal chat -- \n");

    let mut conversation_context = ConversationContext::new();
    let developer_message = Message {
        role: "developer".into(),
        content: "You are helpful, intelligent, and friendly.
You are also very concise and accurate.
No words are wasted in your responses.
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
        match readline {
            Ok(line) => {
                let line = line.trim().to_owned();
                if line.is_empty() {
                    continue;
                }
                rl.add_history_entry(&line).unwrap();

                if line.starts_with(':') {
                    match parse_command(&line) {
                        Command::Quit => break,
                        Command::Clear => {
                            conversation_context.messages.clear();
                            conversation_context
                                .messages
                                .push(developer_message.clone());
                            println!("\nConversation cleared.\n");
                        }
                        Command::Debug => {
                            println!("\nDebugging conversation messages:");
                            for msg in &conversation_context.messages {
                                println!("{}: {}", msg.role, msg.content);
                            }
                            println!();
                        }
                        Command::Doc => chat_client.document(&conversation_context).await?,
                        Command::Unknown(cmd) => {
                            eprintln!("\nInvalid command: {}\n", cmd);
                        }
                    }
                } else {
                    conversation_context.messages.push(Message {
                        role: "user".into(),
                        content: line.clone(),
                    });
                    let response = chat_client.send_request(&conversation_context).await?;
                    if let Some(choice) = response.choices.first() {
                        let reply = choice.message.content.clone();
                        conversation_context.messages.push(Message {
                            role: "assistant".into(),
                            content: reply.clone(),
                        });
                        println!("\n{}\n", reply);
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}

struct ChatClient {
    client: reqwest::Client,
    url: String,
    api_key: String,
}

impl ChatClient {
    fn new() -> Result<Self, reqwest::Error> {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        Ok(Self {
            client: reqwest::Client::new(),
            url: "https://api.openai.com/v1/chat/completions".into(),
            api_key,
        })
    }

    async fn send_request(
        &self,
        context: &ConversationContext,
    ) -> Result<Response, reqwest::Error> {
        let request_json = serde_json::to_string(context).unwrap();
        let response_text = run_with_spinner(async {
            self.client
                .post(&self.url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .body(request_json)
                .send()
                .await?
                .text()
                .await
        })
        .await?;
        print!("\r                   \r");
        let _ = stdout().flush();
        let response_data: Response = serde_json::from_str(&response_text).unwrap();
        Ok(response_data)
    }

    async fn document(&self, context: &ConversationContext) -> Result<(), reqwest::Error> {
        let mut new_context = ConversationContext::new();
        let dev_message = Message {
            role: "developer".into(),
            content:
                "You are a conversation distiller. Your job is to look at the following conversation
and create a well formed document about the topics in the conversation. Do not talk about the
people in the conversations, or that it is a conversation. Extract the meaning and data of
the conversation and put it into a well formed report"
                    .into(),
        };
        new_context.messages.push(dev_message);
        for msg in &context.messages {
            if msg.role != "developer" {
                new_context.messages.push(msg.clone());
            }
        }
        let response = self.send_request(&new_context).await?;
        if let Some(choice) = response.choices.first() {
            println!("{}", choice.message.content);
        }
        Ok(())
    }
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
        Self {
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
