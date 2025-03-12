use std::{
    fs::File,
    io::{Write, stdin, stdout},
    path::Path,
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
        match readline {
            Ok(line) => {
                let line = line.trim().to_owned();
                if line.is_empty() {
                    continue;
                }
                rl.add_history_entry(&line).unwrap();

                match line.as_str() {
                    ":q" => break,
                    ":clear" => {
                        conversation_context.messages.clear();
                        conversation_context
                            .messages
                            .push(developer_message.clone());
                        println!("\nConversation cleared.\n");
                    }
                    ":debug" => {
                        println!("\nDebugging conversation messages:");
                        for msg in &conversation_context.messages {
                            println!("{}: {}", msg.role, msg.content);
                        }
                        println!();
                    }
                    ":doc" => {
                        chat_client.document(&conversation_context).await?;
                    }
                    _ => {
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
        // Generate the report document.
        let mut new_context = ConversationContext::new();
        let dev_message = Message {
            role: "developer".into(),
            content: "Your job is to look at the following conversation
and create a well-formed document about the topics in the conversation. Do not talk about the
people in the conversations, or that it is a conversation. Extract the meaning and data of
the conversation and put it into a well-formed report, do not omit any part of the
conversation. If there is code, please put it in the report.
Make sure the report is written in markdown."
                .into(),
        };
        new_context.messages.push(dev_message);
        for msg in &context.messages {
            if msg.role != "developer" {
                new_context.messages.push(msg.clone());
            }
        }
        let response = self.send_request(&new_context).await?;
        let report = if let Some(choice) = response.choices.first() {
            choice.message.content.clone()
        } else {
            eprintln!("No report content received.");
            return Ok(());
        };

        // Ask the AI for a title for the report.
        let mut title_context = ConversationContext::new();
        let title_prompt = Message {
            role: "developer".into(),
            content: format!(
                "You are an assistant that creates concise titles for reports. Based on the following report content, provide a one-line title that summarizes the content. Do not include any additional text.\n\n{}",
                report
            ),
        };
        title_context.messages.push(title_prompt);
        let title_response = self.send_request(&title_context).await?;
        let title = if let Some(choice) = title_response.choices.first() {
            choice.message.content.trim().to_string()
        } else {
            "Report".to_string()
        };

        // Sanitize the title for use as a file name
        let sanitized_title = title.replace("/", "_").replace("\\", "_");
        if !Path::new("reports").exists() {
            std::fs::create_dir("reports").unwrap();
        }
        let filename = format!("reports/{}.md", sanitized_title);

        // Create file content with the title as the first line, an empty line, and then the report.
        let file_contents = format!("{}\n\n{}", title, report);

        // Display the document on the console.
        println!("\n{}", file_contents);

        // Prompt before saving the document, including the filename.
        println!(
            "\nDo you want to save this document as '{}'? (y/n): ",
            filename
        );
        let mut answer = String::new();
        stdin()
            .read_line(&mut answer)
            .expect("Failed to read input");
        if answer.trim().eq_ignore_ascii_case("y") || answer.trim().eq_ignore_ascii_case("yes") {
            let mut file = File::create(&filename).expect("Could not create file");
            file.write_all(file_contents.as_bytes())
                .expect("Could not write to file");
            println!("\nDocument saved as '{}'\n", filename);
        } else {
            println!("Document not saved.");
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
