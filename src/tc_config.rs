use crate::{
    chat_client::get_models,
    commands::change_model::{AVAILABLE_MODELS, ModelsResponse},
    messages::MESSAGES,
    utils::confirm_action,
};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, io::stdin, path::PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ConfigTC {
    #[serde(default = "default_streaming")]
    pub(crate) enable_streaming: bool,

    #[serde(default = "default_model")]
    pub(crate) model: String,

    #[serde(default = "default_dev_message")]
    pub(crate) dev_message: String,

    #[serde(default)]
    pub(crate) preview_md: bool,
}

fn default_model() -> String {
    String::from("gpt-4o-mini")
}

fn default_dev_message() -> String {
    MESSAGES["developer"].to_string()
}

fn default_streaming() -> bool {
    false
}

pub async fn load_config() -> Result<ConfigTC, Box<dyn Error>> {
    let models_response: ModelsResponse = serde_json::from_str(&get_models().await?)?;

    let names: Vec<String> = models_response.data.into_iter().map(|m| m.id).collect();
    let all_models: Vec<String> = AVAILABLE_MODELS
        .iter()
        .map(|&model| model.to_string())
        .chain(names)
        .collect();

    if let Ok(file) = File::open(get_config_path()) {
        match serde_json::from_reader::<File, ConfigTC>(file) {
            Ok(mut config) => {
                if !all_models.contains(&config.model) {
                    println!(
                        "\nInvalid model found in config. Using default model: {}",
                        default_model()
                    );
                    config.model = default_model();
                }
                Ok(config)
            }
            Err(_) => {
                println!("\nFailed to load config. Using default values.");
                Ok(ConfigTC::default())
            }
        }
    } else if confirm_action("No config file found. Would you like to set one up? (y/n)") {
        let mut config = ConfigTC::default();
        println!("\nAvailable models:");
        for (i, model) in all_models.iter().enumerate() {
            println!("{}) {}", i + 1, model);
        }
        println!("\nPlease select a model by typing its number:");
        let mut model_choice = String::new();
        stdin()
            .read_line(&mut model_choice)
            .expect("failed to read line");

        match model_choice.trim().parse::<usize>() {
            Ok(num) if num > 0 && num <= all_models.len() => {
                config.model = all_models[num - 1].to_string();
            }
            _ => {
                eprintln!(
                    "\nInvalid selection. Using default model: {}",
                    default_model()
                );
                config.model = default_model();
            }
        }

        config.enable_streaming = confirm_action(
            "Would you like to enable streaming for eligible models (experimental)? (y/n)",
        );

        config.preview_md = confirm_action(
            "Would you like to display non-streamed model responses as rendered markdown (experimental)? (y/n)",
        );

        if confirm_action("Would you like to write a custom developer message for the AI? (y/n)") {
            let mut prompt = String::new();
            stdin().read_line(&mut prompt).expect("failed to read line");
            config.dev_message = prompt;
        }

        println!("\n{:#?}\n", config);
        write_config(&config).expect("Error writing config");

        Ok(config)
    } else {
        println!("Using default values.");
        Ok(ConfigTC::default())
    }
}

pub fn write_config(config: &ConfigTC) -> std::io::Result<()> {
    let path = get_config_path();

    if confirm_action(&format!("Save to {}?", path.to_str().unwrap())) {
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
        // Fallback to current directory if the dirs crate fails
        std::env::current_dir().unwrap().join("tc_config.json")
    }
}

impl ConfigTC {
    pub fn default() -> Self {
        Self {
            enable_streaming: default_streaming(),
            model: default_model(),
            dev_message: default_dev_message(),
            preview_md: true,
        }
    }
}
