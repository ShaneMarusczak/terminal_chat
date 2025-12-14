use linefeed::{Interface, ReadResult};

use crate::{
    chat_client::{get_anthropic_models, get_openai_models},
    commands::change_model::ModelsResponse,
    conversation::Response,
};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::{collections::HashSet, error::Error};

pub(crate) fn walk_directory(
    path: &str,
    extensions: &HashSet<&str>,
    excluded_dirs: &HashSet<&str>,
) -> std::io::Result<Vec<(String, String)>> {
    let mut results = Vec::new();

    if Path::new(path).is_dir() {
        visit_files(Path::new(path), extensions, excluded_dirs, &mut results)?;
    }

    Ok(results)
}

fn visit_files(
    path: &Path,
    extensions: &HashSet<&str>,
    excluded_dirs: &HashSet<&str>,
    results: &mut Vec<(String, String)>,
) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str())
                    && excluded_dirs.contains(dir_name) {
                        continue;
                    }
                visit_files(&path, extensions, excluded_dirs, results)?;
            } else if path.is_file() {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !filename.starts_with('.') {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if (extensions.is_empty() || extensions.contains(ext))
                        && let Ok(content) = fs::read_to_string(&path) {
                            results.push((path.display().to_string(), content));
                        }
                }
            }
        }
    }
    Ok(())
}

pub fn calculate_message_width(
    message_text: &str,
    max_chat_width: usize,
    message_width_percent: usize,
) -> (usize, usize) {
    let terminal_width = termsize::get().map(|size| size.cols as usize).unwrap_or(80);
    let max_width = terminal_width.min(max_chat_width) * message_width_percent / 100;

    let lines: Vec<&str> = message_text.lines().collect();
    if lines.len() == 1 {
        ((lines[0].len() + 4).min(max_width), terminal_width) // Add 4 for padding
    } else {
        (max_width, terminal_width)
    }
}

pub fn extract_message_text(response: &Response) -> Option<String> {
    for output in &response.output {
        if output.type_field == "message"
            && let Some(content) = &output.content
                && let Some(first_content) = content.first() {
                    return Some(first_content.text.clone());
                }
    }
    None
}

pub fn read_user_input(prompt: &str) -> Result<String, Box<dyn Error>> {
    let interface = Interface::new("tc")?;
    interface.set_prompt(prompt)?;
    match interface.read_line()? {
        ReadResult::Input(line) => Ok(line.trim().to_string()),
        ReadResult::Eof => Err("End of input (EOF) received".into()),
        ReadResult::Signal(_) => Err("Input interrupted by signal".into()),
    }
}

pub fn confirm_action(prompt: &str) -> bool {
    let response = read_user_input(prompt);
    response.is_ok_and(|c| c.eq_ignore_ascii_case("y"))
}

pub fn select_model(all_models: &[String], prompt_message: &str) -> Result<String, Box<dyn Error>> {
    println!("\n{}", prompt_message);
    println!("Available models:");
    for (i, model) in all_models.iter().enumerate() {
        println!("{}) {}", i + 1, model);
    }

    loop {
        let input = read_user_input("\nSelect a model by number: ")?;
        if let Ok(num) = input.trim().parse::<usize>()
            && num > 0 && num <= all_models.len() {
                return Ok(all_models[num - 1].clone());
            }
        eprintln!("Invalid selection. Please try again.");
    }
}

#[derive(Debug, Deserialize)]
struct OpenAIModel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModel>,
}

pub async fn get_all_model_names(
    anthropic_enabled: bool,
    openai_enabled: bool,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut all_models = Vec::new();

    if openai_enabled {
        let openai_response: OpenAIModelsResponse =
            serde_json::from_str(&get_openai_models().await?)?;
        let openai_names: Vec<String> = openai_response
            .data
            .into_iter()
            .map(|m| m.id)
            .collect();
        all_models.extend(openai_names);
    }

    if anthropic_enabled {
        let anthropic_response: ModelsResponse =
            serde_json::from_str(&get_anthropic_models().await?)?;
        let anthropic_names: Vec<String> = anthropic_response
            .data
            .into_iter()
            .map(|m| m.id)
            .collect();
        all_models.extend(anthropic_names);
    }

    if all_models.is_empty() {
        return Err("No models available".into());
    }

    Ok(all_models)
}

pub(crate) fn sequence_equals(slice1: &[String], slice2: &[String]) -> bool {
    if slice1.len() != slice2.len() {
        return false;
    }

    let set1: HashSet<_> = slice1.iter().collect();
    let set2: HashSet<_> = slice2.iter().collect();

    set1 == set2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_equals() {
        // Equal sequences
        assert!(sequence_equals(
            &["a".to_string(), "b".to_string()],
            &["a".to_string(), "b".to_string()]
        ));

        // Equal sequences in different order
        assert!(sequence_equals(
            &["a".to_string(), "b".to_string()],
            &["b".to_string(), "a".to_string()]
        ));

        // Different lengths
        assert!(!sequence_equals(
            &["a".to_string(), "b".to_string()],
            &["a".to_string(), "b".to_string(), "c".to_string()]
        ));

        // Different contents
        assert!(!sequence_equals(
            &["a".to_string(), "b".to_string()],
            &["a".to_string(), "c".to_string()]
        ));

        // Empty sequences
        assert!(sequence_equals(&[], &[]));
    }
}
