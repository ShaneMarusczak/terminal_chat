use serde::{Deserialize, Serialize};
use std::{fs::File, io::stdin};

use crate::{commands::change_model::AVAILABLE_MODELS, messages::MESSAGES, utils::confirm_action};

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

pub fn load_config() -> ConfigTC {
    if let Ok(file) = File::open("tc_config.json") {
        match serde_json::from_reader::<File, ConfigTC>(file) {
            Ok(mut config) => {
                if !AVAILABLE_MODELS.contains(&config.model.as_str()) {
                    println!(
                        "Invalid model found in config. Using default model: {}",
                        default_model()
                    );
                    config.model = default_model();
                }
                config
            }
            Err(_) => {
                println!("Failed to load config. Using default values.");
                ConfigTC::default()
            }
        }
    } else if confirm_action("No config file found. Would you like to set one up? (y/n)") {
        let mut config = ConfigTC::default();
        println!("\nAvailable models:");
        for (i, model) in AVAILABLE_MODELS.iter().enumerate() {
            println!("{}) {}", i + 1, model);
        }
        println!("\nPlease select a model by typing its number:");
        let mut model_choice = String::new();
        stdin()
            .read_line(&mut model_choice)
            .expect("failed to read line");

        match model_choice.trim().parse::<usize>() {
            Ok(num) if num > 0 && num <= AVAILABLE_MODELS.len() => {
                config.model = AVAILABLE_MODELS[num - 1].to_string();
            }
            _ => {
                eprintln!(
                    "Invalid selection. Using default model: {}",
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
        if confirm_action("Save this config?") {
            write_config(&config).expect("Error writing config");
        }

        config
    } else {
        println!("Using default values.");
        ConfigTC::default()
    }

    // Return default if file not found
}

pub fn write_config(config: &ConfigTC) -> std::io::Result<()> {
    let file = File::create("tc_config.json")?;
    serde_json::to_writer(file, config)?;
    Ok(())
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
