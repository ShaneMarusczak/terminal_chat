use crate::conversation::{ConversationContext, Response};
use crate::messages::MESSAGES;
use crate::spinner::run_with_spinner;
use std::fs::File;
use std::io::{Write, stdout};
use std::path::Path;

pub struct ChatClient {
    client: reqwest::Client,
    pub url: String,
    pub api_key: String,
}

impl ChatClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        Ok(Self {
            client: reqwest::Client::new(),
            url: "https://api.openai.com/v1/chat/completions".into(),
            api_key,
        })
    }

    pub async fn send_request(
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

    pub async fn document(&self, context: &ConversationContext) -> Result<(), reqwest::Error> {
        let mut new_context = ConversationContext::new("o3-mini");
        let dev_message = crate::conversation::Message {
            role: "developer".into(),
            content: MESSAGES.get("document_prompt").unwrap().to_string(),
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
        let mut title_context = ConversationContext::new("gpt-4o");
        let title_prompt = crate::conversation::Message {
            role: "developer".into(),
            content: format!(
                "{} \n::\n {}",
                MESSAGES.get("title_prompt").unwrap(),
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
        let sanitized_title = title.replace("/", "_").replace("\\", "_").replace('"', "");
        if !Path::new("reports").exists() {
            std::fs::create_dir("reports").unwrap();
        }
        let filename = format!("reports/{}.md", sanitized_title);
        let file_contents = format!("{}\n\n{}", title, report);
        println!("\n{}", file_contents);
        println!(
            "\nDo you want to save this document as '{}'? (y/n): ",
            filename
        );
        let mut answer = String::new();
        std::io::stdin()
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
