use std::{
    io::{Write, stdin, stdout},
    process::exit,
};

use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let mut session = Session { messages: vec![] };

    let dev_message = Message {
        role: String::from("developer"),
        content: "You are helpful, intelligent, and friendly.
                You are also very concise and accurate.
                No words are wasted in your responses.
                Always answer with very accurate and kind responses that are short, to the point and friendly."
            .to_owned(),
    };

    session.messages.push(dev_message.clone());

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
                    session.messages.clear();
                    session.messages.push(dev_message.clone());
                    println!("Session cleared.");
                }
                "debug" => {
                    println!("Debugging session messages:");
                    for message in &session.messages {
                        println!("{}: {}", message.role, message.content);
                    }
                }
                _ => continue,
            }
        } else {
            let user_message = Message {
                role: String::from("user"),
                content: line.clone(),
            };
            session.messages.push(user_message);

            let request = make_request(&session);

            let request_as_json = serde_json::to_string(&request).unwrap();

            let test = client
                .post(url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", api_key))
                .body(request_as_json)
                .send()
                .await?
                .text()
                .await?;

            let response_data: Response = serde_json::from_str(&test).unwrap();

            if let Some(choice) = response_data.choices.first() {
                let content = &choice.message.content;
                let ai_message = Message {
                    role: String::from("assistant"),
                    content: String::from(content),
                };
                session.messages.push(ai_message);
                println!("{}", content);
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

fn make_request(session: &Session) -> Request {
    let mut request = Request {
        model: String::new(),
        messages: vec![],
    };

    request.model = String::from("gpt-4o");

    for message in session.messages.clone() {
        request.messages.push(Message {
            role: message.role,
            content: message.content.clone(),
        });
    }

    request
}

struct Session {
    messages: Vec<Message>,
}

#[derive(serde::Serialize, Deserialize, Clone, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(serde::Serialize, Debug)]
struct Request {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Choice {
    message: Message,
}
