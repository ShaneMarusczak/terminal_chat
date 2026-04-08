use linefeed::{Interface, ReadResult};

use crate::{
    chat_client::{get_anthropic_models, get_openai_models},
    commands::change_model::ModelsResponse,
    conversation::Provider,
};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::Path;

pub fn walk_directory(
    path: &str,
    extensions: &HashSet<&str>,
    excluded_dirs: &HashSet<&str>,
) -> std::io::Result<Vec<(String, String)>> {
    let mut results = Vec::new();
    let root = Path::new(path);
    if root.is_dir() {
        visit_files(root, extensions, excluded_dirs, &mut results)?;
    }
    Ok(results)
}

fn visit_files(
    path: &Path,
    extensions: &HashSet<&str>,
    excluded_dirs: &HashSet<&str>,
    results: &mut Vec<(String, String)>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let name = entry_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if entry_path.is_dir() {
            if !excluded_dirs.contains(name) {
                visit_files(&entry_path, extensions, excluded_dirs, results)?;
            }
        } else if entry_path.is_file() && !name.starts_with('.') {
            let ext = entry_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if (extensions.is_empty() || extensions.contains(ext))
                && let Ok(content) = fs::read_to_string(&entry_path)
            {
                results.push((entry_path.display().to_string(), content));
            }
        }
    }
    Ok(())
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

/// Print a numbered, provider-grouped model list.
pub fn print_model_list(models: &[String]) {
    let mut current: Option<Provider> = None;
    for (index, model) in models.iter().enumerate() {
        let provider = Provider::from_model_name(model);
        if Some(provider) != current {
            if current.is_some() {
                println!();
            }
            println!("{} Models:", provider_label(provider));
            current = Some(provider);
        }
        println!("{}) {}", index + 1, model);
    }
}

fn provider_label(p: Provider) -> &'static str {
    match p {
        Provider::Anthropic => "Anthropic",
        Provider::OpenAI => "OpenAI",
        Provider::Local => "Local",
    }
}

/// Prompt the user to pick a model by number. Returns the selected model name.
pub fn prompt_model_selection(models: &[String]) -> Result<String, Box<dyn Error>> {
    loop {
        let input = read_user_input("\nSelect a model by number: ")?;
        if let Ok(num) = input.trim().parse::<usize>()
            && (1..=models.len()).contains(&num)
        {
            return Ok(models[num - 1].clone());
        }
        eprintln!("Invalid selection. Please try again.");
    }
}

pub fn select_model(all_models: &[String], prompt_message: &str) -> Result<String, Box<dyn Error>> {
    println!("\n{}", prompt_message);
    print_model_list(all_models);
    prompt_model_selection(all_models)
}

// Substring/prefix tables for `filter_models`. Kept here so additions are
// trivial: drop a string in the matching slice.
const EXCLUDED_PREFIXES: &[&str] = &[
    "ada", "babbage", "curie", "davinci", "text-", "code-", "gpt-3",
];

const EXCLUDED_SUBSTRINGS: &[&str] = &[
    "instruct",
    "transcribe",
    "audio",
    "whisper",
    "tts",
    "sora",
    "omni",
    "realtime",
    "codex",
    "dall",
    "preview",
    "nano",
    "turbo",
    "latest",
    "search-api",
    "deep-research",
    "image",
    "mini",
    "pro",
];

/// Drop fine-tuned models, deprecated OpenAI families, and specialized
/// (non-chat) models like audio/image/realtime variants.
pub fn filter_models(models: Vec<String>) -> Vec<String> {
    models
        .into_iter()
        .filter(|m| {
            if m.contains(':') {
                return false;
            }
            let lower = m.to_lowercase();
            !EXCLUDED_PREFIXES.iter().any(|p| lower.starts_with(p))
                && !EXCLUDED_SUBSTRINGS.iter().any(|s| lower.contains(s))
        })
        .collect()
}

/// Splits a model name like "claude-3-5-sonnet-20241022" into
/// `("claude-3-5-sonnet", "20241022")`. Returns `None` for dateless names.
fn split_date_suffix(model: &str) -> Option<(&str, &str)> {
    let (base, suffix) = model.rsplit_once('-')?;
    let looks_like_date = suffix.starts_with("20")
        && suffix.len() >= 8
        && suffix.chars().all(|c| c.is_ascii_digit() || c == '-');
    if looks_like_date {
        Some((base, suffix))
    } else {
        None
    }
}

/// Returns the base model name (the part before any trailing date suffix).
pub fn get_base_model_name(model: &str) -> &str {
    split_date_suffix(model).map(|(base, _)| base).unwrap_or(model)
}

/// True if the model name ends in a date suffix.
pub fn has_date_suffix(model: &str) -> bool {
    split_date_suffix(model).is_some()
}

/// Group models by base name and pick one per group: a dateless version if
/// present, otherwise the first (newest) dated version after sorting.
pub fn deduplicate_models(models: Vec<String>) -> Vec<String> {
    let mut by_base: HashMap<String, Vec<String>> = HashMap::new();
    for model in models {
        let base = get_base_model_name(&model).to_string();
        by_base.entry(base).or_default().push(model);
    }

    by_base
        .into_values()
        .filter_map(|versions| {
            versions
                .iter()
                .find(|m| !has_date_suffix(m))
                .or_else(|| versions.first())
                .cloned()
        })
        .collect()
}

/// Sort by family preference: Sonnet > GPT-4o > GPT-4 > Opus > alphabetical
/// (descending so newer dates float to the top).
fn sort_models(models: &mut [String]) {
    fn rank(name: &str) -> u8 {
        let lower = name.to_lowercase();
        if lower.contains("sonnet") {
            0
        } else if lower.starts_with("gpt-4o") {
            1
        } else if lower.starts_with("gpt-4") {
            2
        } else if lower.contains("opus") {
            3
        } else {
            4
        }
    }

    models.sort_by(|a, b| rank(a).cmp(&rank(b)).then_with(|| b.cmp(a)));
}

pub fn get_default_model(models: &[String], anthropic_enabled: bool) -> String {
    if anthropic_enabled
        && let Some(sonnet) = models.iter().find(|m| m.to_lowercase().contains("sonnet"))
    {
        return sonnet.clone();
    }

    if let Some(gpt) = models.iter().find(|m| {
        let lower = m.to_lowercase();
        lower.starts_with("gpt") && !m.contains(':')
    }) {
        return gpt.clone();
    }

    models
        .first()
        .cloned()
        .unwrap_or_else(|| "default_model_name".to_string())
}

/// Parse a `/v1/models` JSON response, then filter, sort, and dedupe.
fn process_models(json: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let response: ModelsResponse = serde_json::from_str(json)?;
    let ids: Vec<String> = response.data.into_iter().map(|m| m.id).collect();
    let mut filtered = filter_models(ids);
    sort_models(&mut filtered);
    Ok(deduplicate_models(filtered))
}

pub async fn get_all_model_names(
    anthropic_enabled: bool,
    openai_enabled: bool,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut all_models = Vec::new();

    if anthropic_enabled {
        all_models.extend(process_models(&get_anthropic_models().await?)?);
    }
    if openai_enabled {
        all_models.extend(process_models(&get_openai_models().await?)?);
    }

    if all_models.is_empty() {
        return Err("No models available".into());
    }
    Ok(all_models)
}

pub fn sequence_equals(slice1: &[String], slice2: &[String]) -> bool {
    if slice1.len() != slice2.len() {
        return false;
    }

    let set1: HashSet<_> = slice1.iter().collect();
    let set2: HashSet<_> = slice2.iter().collect();

    set1 == set2
}

