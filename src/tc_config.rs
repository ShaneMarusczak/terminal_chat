use crate::{
    messages::MESSAGES,
    utils::{confirm_action, get_default_model, print_model_list, prompt_model_selection, read_user_input, sequence_equals},
};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{env, error::Error, fs::File, path::PathBuf, sync::LazyLock};

use std::sync::RwLock;

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

    if !anthropic_enabled && !openai_enabled {
        return Err(
            "No API keys detected.\n\
            You must set at least one of the following environment variables:\n\
            - ANTHROPIC_API_KEY (for Claude models)\n\
            - OPENAI_API_KEY (for GPT models)"
                .into(),
        );
    }

    let all_models = crate::utils::get_all_model_names(anthropic_enabled, openai_enabled).await?;

    let rv = if let Ok(file) = File::open(get_config_path()) {
        match serde_json::from_reader::<File, ConfigTC>(file) {
            Ok(mut config) => {
                if !sequence_equals(&config.all_models, &all_models) {
                    config.all_models = all_models.clone();
                    // Update the configuration file with the new models list
                    write_config(&config, false)?;
                }
                if !all_models.contains(&config.model) {
                    let default = get_default_model(&all_models, anthropic_enabled);
                    eprintln!("\nInvalid model found in config. Using: {}", default);
                    config.model = default;
                }
                config
            }
            Err(_) => {
                println!("\nFailed to load config. Using default values.");
                ConfigTC::default(all_models)
            }
        }
    } else if confirm_action("No config file found. Would you like to set one up? (y/n)") {
        let mut config = ConfigTC::default(all_models.clone());
        config_interview(&mut config);
        write_config(&config, true)?;
        config
    } else {
        println!("Using default values.");
        ConfigTC::default(all_models)
    };

    let mut global = GLOBAL_CONFIG.write()?;
    *global = rv.clone();
    Ok(rv)
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

