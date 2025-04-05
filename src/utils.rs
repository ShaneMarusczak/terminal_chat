use linefeed::{Interface, ReadResult};

use crate::{
    chat_client::get_models, commands::change_model::ModelsResponse, conversation::Response,
};
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
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if excluded_dirs.contains(dir_name) {
                        continue;
                    }
                }
                visit_files(&path, extensions, excluded_dirs, results)?;
            } else if path.is_file() {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !filename.starts_with('.') {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if extensions.is_empty() || extensions.contains(ext) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            results.push((path.display().to_string(), content));
                        }
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
) -> usize {
    let terminal_width = termsize::get().map(|size| size.cols as usize).unwrap_or(80);
    let max_width = terminal_width.min(max_chat_width) * message_width_percent / 100;

    let lines: Vec<&str> = message_text.lines().collect();
    if lines.len() == 1 {
        (lines[0].len() + 4).min(max_width) // Add 4 for padding
    } else {
        max_width
    }
}

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
    let interface = Interface::new("tc").expect("error making interface");
    interface.set_prompt(prompt).unwrap();
    if let ReadResult::Input(line) = interface.read_line().unwrap() {
        return line.trim().to_string();
    }
    unreachable!()
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
