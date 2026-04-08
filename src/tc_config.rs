use crate::{
    chat_client::get_local_models,
    commands::change_model::ModelsResponse,
    messages::MESSAGES,
    utils::{confirm_action, get_default_model, print_model_list, prompt_model_selection, read_user_input, sequence_equals},
};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{env, error::Error, fs::File, path::PathBuf, sync::LazyLock};

use std::sync::RwLock;

const DEFAULT_LOCAL_URL: &str = "http://localhost:8000/v1";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigTC {
    #[serde(default)]
    pub model: String,

    #[serde(default)]
    pub all_models: Vec<String>,

    #[serde(default = "default_dev_message")]
    pub dev_message: String,

    #[serde(default = "default_anthropic")]
    pub anthropic_enabled: bool,

    #[serde(default = "default_openai")]
    pub openai_enabled: bool,

    #[serde(default = "default_theme")]
    pub theme: Theme,

    #[serde(default)]
    pub local_base_url: Option<String>,
}

pub(crate) static GLOBAL_CONFIG: LazyLock<RwLock<ConfigTC>> =
    LazyLock::new(|| RwLock::new(ConfigTC::default(vec![])));

fn default_dev_message() -> String {
    MESSAGES["developer"].to_string()
}

fn default_anthropic() -> bool {
    env::var("ANTHROPIC_API_KEY").is_ok()
}

fn default_openai() -> bool {
    env::var("OPENAI_API_KEY").is_ok()
}

pub async fn load_config() -> Result<ConfigTC, Box<dyn Error>> {
    let anthropic_enabled = default_anthropic();
    let openai_enabled = default_openai();
    let any_remote = anthropic_enabled || openai_enabled;

    // No remote keys: try to honor an existing local-only config, otherwise
    // offer the local setup wizard.
    if !any_remote {
        if let Some(config) = try_load_local_only_config() {
            let mut global = GLOBAL_CONFIG.write()?;
            *global = config.clone();
            return Ok(config);
        }
        return offer_local_or_exit().await;
    }

    let remote_models =
        crate::utils::get_all_model_names(anthropic_enabled, openai_enabled).await?;

    let rv = if let Ok(file) = File::open(get_config_path()) {
        match serde_json::from_reader::<File, ConfigTC>(file) {
            Ok(mut config) => {
                // Preserve any local/* entries the user added previously.
                let local_entries: Vec<String> = config
                    .all_models
                    .iter()
                    .filter(|m| m.starts_with("local/"))
                    .cloned()
                    .collect();
                let merged: Vec<String> = remote_models
                    .iter()
                    .cloned()
                    .chain(local_entries.into_iter())
                    .collect();

                if !sequence_equals(&config.all_models, &merged) {
                    config.all_models = merged;
                    // Update the configuration file with the new models list
                    write_config(&config, false)?;
                }
                if !config.all_models.contains(&config.model) {
                    let default = get_default_model(&config.all_models, anthropic_enabled);
                    eprintln!("\nInvalid model found in config. Using: {}", default);
                    config.model = default;
                }
                config
            }
            Err(_) => {
                println!("\nFailed to load config. Using default values.");
                ConfigTC::default(remote_models)
            }
        }
    } else if confirm_action("No config file found. Would you like to set one up? (y/n)") {
        let mut config = ConfigTC::default(remote_models.clone());
        config_interview(&mut config);
        write_config(&config, true)?;
        config
    } else {
        println!("Using default values.");
        ConfigTC::default(remote_models)
    };

    let mut global = GLOBAL_CONFIG.write()?;
    *global = rv.clone();
    Ok(rv)
}

/// Try to load an existing config file that's already set up for local-only
/// use (has `local_base_url` set or `TC_LOCAL_URL` in env, plus at least one
/// `local/` model). Returns `None` if there's no such config.
fn try_load_local_only_config() -> Option<ConfigTC> {
    let file = File::open(get_config_path()).ok()?;
    let config: ConfigTC = serde_json::from_reader(file).ok()?;
    let has_local_url = config.local_base_url.is_some() || env::var("TC_LOCAL_URL").is_ok();
    let has_local_model = config.all_models.iter().any(|m| m.starts_with("local/"));
    if has_local_url && has_local_model {
        Some(config)
    } else {
        None
    }
}

/// Interactive recovery when no API keys are set: offer to run the local
/// setup wizard, or exit with instructions for setting an API key.
async fn offer_local_or_exit() -> Result<ConfigTC, Box<dyn Error>> {
    println!();
    println!("No API keys detected.");
    println!();
    println!("Terminal Chat needs at least one model provider. Choose one:");
    println!("  1) Set up a local model endpoint now (oMLX, LM Studio, Ollama, llama.cpp, ...)");
    println!("  2) Quit and set ANTHROPIC_API_KEY or OPENAI_API_KEY");
    println!();

    let choice = read_user_input("Choice [1-2]: ")?;
    match choice.trim() {
        "1" => {
            let config = local_setup_wizard().await?;
            write_config(&config, false)?;
            let mut global = GLOBAL_CONFIG.write()?;
            *global = config.clone();
            Ok(config)
        }
        _ => Err(
            "No API keys detected.\n\
            You must set at least one of the following environment variables:\n\
            - ANTHROPIC_API_KEY (for Claude models)\n\
            - OPENAI_API_KEY (for GPT models)\n\
            Or re-run and choose option 1 to set up a local endpoint."
                .into(),
        ),
    }
}

/// Multi-step wizard for configuring a local OpenAI-compatible endpoint.
/// Probes `/v1/models` to auto-populate the picker; falls back to manual
/// entry if the probe fails.
async fn local_setup_wizard() -> Result<ConfigTC, Box<dyn Error>> {
    println!();
    let url_input = read_user_input(&format!(
        "Local endpoint URL [{}]: ",
        DEFAULT_LOCAL_URL
    ))?;
    let base_url = if url_input.trim().is_empty() {
        DEFAULT_LOCAL_URL.to_string()
    } else {
        url_input.trim().trim_end_matches('/').to_string()
    };

    println!("\nProbing {}/models ...", base_url);
    let selected = match get_local_models(&base_url).await {
        Ok(json) => match parse_local_model_ids(&json) {
            Ok(ids) if !ids.is_empty() => {
                println!("Found {} model(s):", ids.len());
                for (i, id) in ids.iter().enumerate() {
                    println!("  {}) {}", i + 1, id);
                }
                let input = read_user_input(
                    "\nSelect models to enable (comma-separated, e.g. 1,3 — or 'all'): ",
                )?;
                parse_selection(&input, &ids)?
            }
            Ok(_) => {
                println!("Server returned no models. Falling back to manual entry.");
                manual_model_entry()?
            }
            Err(e) => {
                println!("Couldn't parse model list ({}). Falling back to manual entry.", e);
                manual_model_entry()?
            }
        },
        Err(e) => {
            println!(
                "Couldn't reach {} ({}) — check the URL if this is unexpected.",
                base_url, e
            );
            println!("Falling back to manual entry.");
            manual_model_entry()?
        }
    };

    if selected.is_empty() {
        return Err("At least one local model must be enabled".into());
    }

    let prefixed: Vec<String> = selected
        .into_iter()
        .map(|m| {
            if m.starts_with("local/") {
                m
            } else {
                format!("local/{}", m)
            }
        })
        .collect();

    let default_model = prefixed[0].clone();
    println!("\nEnabled: {}", prefixed.join(", "));
    println!("Default model: {}", default_model);

    Ok(ConfigTC {
        model: default_model,
        all_models: prefixed,
        dev_message: default_dev_message(),
        anthropic_enabled: false,
        openai_enabled: false,
        theme: default_theme(),
        local_base_url: Some(base_url),
    })
}

/// Parse model IDs from an OpenAI-compatible `/v1/models` response. No
/// filtering — local model names often contain substrings like "instruct"
/// that the remote model filter would strip.
fn parse_local_model_ids(json: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let response: ModelsResponse = serde_json::from_str(json)?;
    Ok(response.data.into_iter().map(|m| m.id).collect())
}

/// Parse a comma-separated user selection like "1,3,4" or "all" into the
/// corresponding model names.
fn parse_selection(input: &str, names: &[String]) -> Result<Vec<String>, Box<dyn Error>> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("No selection provided".into());
    }
    if trimmed.eq_ignore_ascii_case("all") {
        return Ok(names.to_vec());
    }
    let mut selected = Vec::new();
    for part in trimmed.split(',') {
        let token = part.trim();
        if token.is_empty() {
            continue;
        }
        let n: usize = token
            .parse()
            .map_err(|_| format!("Invalid selection: '{}'", token))?;
        if n == 0 || n > names.len() {
            return Err(format!("Selection {} out of range (1-{})", n, names.len()).into());
        }
        let name = names[n - 1].clone();
        if !selected.contains(&name) {
            selected.push(name);
        }
    }
    if selected.is_empty() {
        return Err("No valid selections parsed".into());
    }
    Ok(selected)
}

/// Manual fallback: prompt for model names one per line until blank input.
fn manual_model_entry() -> Result<Vec<String>, Box<dyn Error>> {
    println!("\nEnter model names one per line. Blank line when done.");
    let mut names = Vec::new();
    loop {
        let prompt = format!("Model {}: ", names.len() + 1);
        let input = read_user_input(&prompt)?;
        let trimmed = input.trim();
        if trimmed.is_empty() {
            if names.is_empty() {
                println!("At least one model required.");
                continue;
            }
            return Ok(names);
        }
        names.push(trimmed.to_string());
    }
}

/// Lightweight config loader: reads JSON file and populates GLOBAL_CONFIG
/// without network calls or API key checks. Used by CLI subcommands like `tc edit`.
pub fn load_config_file() -> Result<ConfigTC, Box<dyn Error>> {
    let config = if let Ok(file) = File::open(get_config_path()) {
        serde_json::from_reader::<File, ConfigTC>(file).unwrap_or_else(|_| ConfigTC::default(vec![]))
    } else {
        ConfigTC::default(vec![])
    };
    let mut global = GLOBAL_CONFIG.write()?;
    *global = config.clone();
    Ok(config)
}

pub fn get_config() -> Result<ConfigTC, Box<dyn Error>> {
    match GLOBAL_CONFIG.read() {
        Ok(gc) => Ok(gc.clone()),
        Err(_) => Err("💩".into()), // Emoji stays per user request!
    }
}

pub fn write_config(config: &ConfigTC, prompt: bool) -> Result<(), Box<dyn Error>> {
    let path = get_config_path();
    if !prompt
        || confirm_action(&format!(
            "Save to {}?",
            path.to_str()
                .ok_or_else(|| format!("Failed to convert path to string: {:?}", path))?
        ))
    {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        serde_json::to_writer(file, config)?;
    }
    Ok(())
}

pub(crate) fn get_config_path() -> PathBuf {
    if let Some(mut config_dir) = config_dir() {
        config_dir.push("tc");
        config_dir.push("tc_config.json");
        config_dir
    } else {
        match std::env::current_dir() {
            Ok(dir) => dir.join("tc_config.json"),
            Err(_) => PathBuf::from("tc_config.json"),
        }
    }
}

impl ConfigTC {
    pub fn default(all_models: Vec<String>) -> Self {
        let anthropic_enabled = default_anthropic();
        let default_model = get_default_model(&all_models, anthropic_enabled);
        Self {
            model: default_model,
            all_models,
            dev_message: default_dev_message(),
            anthropic_enabled,
            openai_enabled: default_openai(),
            theme: default_theme(),
            local_base_url: None,
        }
    }
}

pub fn config_interview(config: &mut ConfigTC) {
    print_model_list(&config.all_models);
    config.model = prompt_model_selection(&config.all_models).unwrap_or_else(|_| config.model.clone());

    if confirm_action("Write a custom developer message for the AI? (y/n)") {
        config.dev_message =
            read_user_input("Enter your custom message:").unwrap_or_else(|_| default_dev_message());
    }

    if confirm_action("Configure a local model endpoint? (y/n)") {
        let url = read_user_input("Local endpoint URL (e.g. http://localhost:1234/v1): ")
            .unwrap_or_default();
        if !url.trim().is_empty() {
            config.local_base_url = Some(url.trim().to_string());
            println!("Local endpoint set. Use model name prefix 'local/' to route there.");
        }
    }
}

pub(crate) fn print_config(config: &ConfigTC) {
    println!(
        "\nConfiguration:\nModel: {}\nDeveloper Message:\n {}\nTheme Colors: System: {}, User: {}, Assistant: {}",
        config.model,
        config.dev_message,
        config.theme.system_color,
        config.theme.user_color,
        config.theme.assistant_color
    );
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Theme {
    #[serde(default = "default_system_color")]
    pub system_color: String,

    #[serde(default = "default_user_color")]
    pub user_color: String,

    #[serde(default = "default_assistant_color")]
    pub assistant_color: String,
}

fn default_system_color() -> String {
    "yellow".to_string()
}

fn default_user_color() -> String {
    "green".to_string()
}

fn default_assistant_color() -> String {
    "blue".to_string()
}

fn default_theme() -> Theme {
    Theme {
        system_color: default_system_color(),
        user_color: default_user_color(),
        assistant_color: default_assistant_color(),
    }
}

