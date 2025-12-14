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

    let mut current_provider: Option<&str> = None;
    let mut display_number = 1;

    for model in all_models {
        // Determine provider
        let provider = if model.to_lowercase().contains("claude") {
            "Anthropic"
        } else {
            "OpenAI"
        };

        // Print provider header if changed
        if current_provider != Some(provider) {
            if current_provider.is_some() {
                println!(); // Add spacing between providers
            }
            println!("{} Models:", provider);
            current_provider = Some(provider);
        }

        println!("{}) {}", display_number, model);
        display_number += 1;
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

/// Filters out unwanted models from the list
fn filter_models(models: Vec<String>) -> Vec<String> {
    models
        .into_iter()
        .filter(|m| {
            let m_lower = m.to_lowercase();

            // Exclude fine-tuned models (contain :)
            if m.contains(':') {
                return false;
            }

            // Exclude old/deprecated OpenAI models
            if m_lower.starts_with("ada")
                || m_lower.starts_with("babbage")
                || m_lower.starts_with("curie")
                || m_lower.starts_with("davinci")
                || m_lower.starts_with("text-")
                || m_lower.starts_with("code-")
                || m_lower.contains("instruct")
            {
                return false;
            }

            // Exclude specialized models not for general chat
            if m_lower.contains("transcribe")
                || m_lower.contains("audio")
                || m_lower.contains("whisper")
                || m_lower.contains("tts")
                || m_lower.contains("sora")
                || m_lower.contains("omni")
                || m_lower.contains("realtime")
                || m_lower.contains("codex")
                || m_lower.contains("dall")
                || m_lower.contains("preview")
                || m_lower.contains("nano")
                || m_lower.contains("turbo")
            {
                return false;
            }

            true
        })
        .collect()
}

/// Extracts the base model name without date suffix
fn get_base_model_name(model: &str) -> &str {
    // For models like "gpt-4o-2024-11-20" or "claude-3-5-sonnet-20241022"
    // Extract the base name by removing the date suffix

    // Find the last date-like pattern (YYYY-MM-DD or YYYYMMDD)
    let parts: Vec<&str> = model.rsplitn(2, '-').collect();
    if parts.len() == 2 {
        let potential_date = parts[0];
        // Check if it looks like a date (starts with 20 and has 8 or 10 chars with digits/hyphens)
        if potential_date.starts_with("20")
            && potential_date.len() >= 8
            && potential_date.chars().all(|c| c.is_ascii_digit() || c == '-') {
            return parts[1];
        }
    }

    model
}

/// Checks if a model has a date suffix
fn has_date_suffix(model: &str) -> bool {
    let parts: Vec<&str> = model.rsplitn(2, '-').collect();
    if parts.len() == 2 {
        let potential_date = parts[0];
        return potential_date.starts_with("20")
            && potential_date.len() >= 8
            && potential_date.chars().all(|c| c.is_ascii_digit() || c == '-');
    }
    false
}

/// Deduplicates models, preferring dateless versions over dated ones
fn deduplicate_models(models: Vec<String>) -> Vec<String> {
    use std::collections::HashMap;

    let mut base_to_models: HashMap<String, Vec<String>> = HashMap::new();

    // Group models by base name
    for model in models {
        let base = get_base_model_name(&model).to_string();
        base_to_models.entry(base).or_default().push(model);
    }

    // For each base, prefer dateless version, otherwise newest dated version
    let mut result = Vec::new();
    for (_base, versions) in base_to_models {
        // Find dateless version
        if let Some(dateless) = versions.iter().find(|m| !has_date_suffix(m)) {
            result.push(dateless.clone());
        } else {
            // All have dates, use the first one (newest due to sorting)
            if let Some(newest) = versions.first() {
                result.push(newest.clone());
            }
        }
    }

    result
}

/// Sorts models with preferred models first
fn sort_models(models: &mut [String]) {
    models.sort_by(|a, b| {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        // Prioritize Claude Sonnet models
        let a_sonnet = a_lower.contains("sonnet");
        let b_sonnet = b_lower.contains("sonnet");
        if a_sonnet != b_sonnet {
            return b_sonnet.cmp(&a_sonnet);
        }

        // Then GPT-4o models
        let a_gpt4o = a_lower.starts_with("gpt-4o");
        let b_gpt4o = b_lower.starts_with("gpt-4o");
        if a_gpt4o != b_gpt4o {
            return b_gpt4o.cmp(&a_gpt4o);
        }

        // Then other GPT-4 models
        let a_gpt4 = a_lower.starts_with("gpt-4");
        let b_gpt4 = b_lower.starts_with("gpt-4");
        if a_gpt4 != b_gpt4 {
            return b_gpt4.cmp(&a_gpt4);
        }

        // Then Claude Opus
        let a_opus = a_lower.contains("opus");
        let b_opus = b_lower.contains("opus");
        if a_opus != b_opus {
            return b_opus.cmp(&a_opus);
        }

        // Finally, sort alphabetically (reverse to get newer dates first)
        b.cmp(a)
    });
}

/// Selects the best default model based on available models
fn select_default_model(models: &[String], anthropic_enabled: bool) -> String {
    // If Anthropic is enabled, prefer newest Sonnet
    if anthropic_enabled
        && let Some(sonnet) = models.iter().find(|m| m.to_lowercase().contains("sonnet")) {
            return sonnet.clone();
        }

    // Otherwise, prefer newest GPT base model
    if let Some(gpt) = models.iter().find(|m| {
        let m_lower = m.to_lowercase();
        m_lower.starts_with("gpt") && !m.contains(':')
    }) {
        return gpt.clone();
    }

    // Fallback to first model in list
    models.first().unwrap_or(&"default_model_name".to_string()).clone()
}

pub async fn get_all_model_names(
    anthropic_enabled: bool,
    openai_enabled: bool,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut anthropic_models = Vec::new();
    let mut openai_models = Vec::new();

    if anthropic_enabled {
        let anthropic_response: ModelsResponse =
            serde_json::from_str(&get_anthropic_models().await?)?;
        let mut models: Vec<String> = anthropic_response
            .data
            .into_iter()
            .map(|m| m.id)
            .collect();
        models = filter_models(models);
        sort_models(&mut models);
        anthropic_models = deduplicate_models(models);
    }

    if openai_enabled {
        let openai_response: OpenAIModelsResponse =
            serde_json::from_str(&get_openai_models().await?)?;
        let mut models: Vec<String> = openai_response
            .data
            .into_iter()
            .map(|m| m.id)
            .collect();
        models = filter_models(models);
        sort_models(&mut models);
        openai_models = deduplicate_models(models);
    }

    // Combine: Anthropic first, then OpenAI
    let mut all_models = Vec::new();
    all_models.extend(anthropic_models);
    all_models.extend(openai_models);

    if all_models.is_empty() {
        return Err("No models available".into());
    }

    Ok(all_models)
}

pub fn get_default_model(all_models: &[String], anthropic_enabled: bool) -> String {
    select_default_model(all_models, anthropic_enabled)
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
