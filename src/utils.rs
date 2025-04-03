use std::{error::Error, io::stdin};

use crate::{
    chat_client::get_models, commands::change_model::ModelsResponse, conversation::Response,
};

pub fn extract_message_text(response: &Response) -> Option<String> {
    for output in &response.output {
        if output.type_field == "message" {
            if let Some(content) = &output.content {
                if let Some(first_content) = content.first() {
                    return Some(first_content.text.clone());
                }
            }
        }
    }
    None
}

pub fn read_user_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

pub fn confirm_action(prompt: &str) -> bool {
    let response = read_user_input(prompt);
    response.eq_ignore_ascii_case("y") || response.eq_ignore_ascii_case("yes")
}

const OPENAI_MODELS: &[&str] = &[
    "gpt-4o",
    "gpt-4o-mini",
    "gpt-4o-search-preview",
    "o1",
    "o3-mini",
];

pub async fn get_all_model_names(
    anthropic_enabled: bool,
    openai_enabled: bool,
) -> Result<Vec<String>, Box<dyn Error>> {
    let models_response: ModelsResponse = if anthropic_enabled {
        serde_json::from_str(&get_models().await.unwrap()).unwrap()
    } else {
        ModelsResponse { data: vec![] }
    };

    let names: Vec<String> = models_response.data.into_iter().map(|m| m.id).collect();
    if openai_enabled && anthropic_enabled {
        Ok(OPENAI_MODELS
            .iter()
            .map(|&model| model.to_string())
            .chain(names)
            .collect())
    } else if openai_enabled {
        Ok(OPENAI_MODELS.iter().map(|m| m.to_string()).collect())
    } else if anthropic_enabled {
        Ok(names)
    } else {
        unreachable!()
    }
}
