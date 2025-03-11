use std::{
    io::{Write, stdin, stdout},
    process::exit,
};

use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let mut conversation_context = ConversationContext::new();

    let dev_message = Message {
        role: String::from("developer"),
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

    loop {
        let line = get_text_input(">> ");

        if line.is_empty() {
            continue;
        } else if let Some(stripped) = line.strip_prefix(':') {
            match stripped {
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
                _ => {
                    eprintln!("\nInvalid command: {}\n", stripped);
                }
            }
        } else {
            let user_message = Message {
                role: String::from("user"),
                content: line.clone(),
            };
            conversation_context.messages.push(user_message);

            let request_as_json = serde_json::to_string(&conversation_context).unwrap();

            let response = client
                .post(url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", api_key))
                .body(request_as_json)
                .send()
                .await?
                .text()
                .await?;

            let response_data: Response = serde_json::from_str(&response).unwrap();

            if let Some(choice) = response_data.choices.first() {
                let content = &choice.message.content;
                let ai_message = Message {
                    role: String::from("assistant"),
                    content: content.to_owned(),
                };
                conversation_context.messages.push(ai_message);
                println!("\n{}\n", content);
            }
        }
    }

    Ok(())
}

fn get_text_input(msg: &str) -> String {
    let mut input = String::new();
    print!("{}", msg);
    let _ = stdout().flush();
    let readline = stdin().read_line(&mut input);
    match readline {
        Ok(_) => input.trim().to_owned(),
        Err(err) => {
            eprintln!("Error: {err:?}");
            exit(0);
        }
    }
}

#[derive(serde::Serialize, Deserialize, Clone, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(serde::Serialize, Debug)]
struct ConversationContext {
    model: String,
    messages: Vec<Message>,
}

impl ConversationContext {
    fn new() -> Self {
        ConversationContext {
            model: String::from("gpt-4o"),
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
