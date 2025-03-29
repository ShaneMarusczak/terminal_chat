use serde::{Deserialize, Serialize};
use std::fs::File;

use crate::{commands::change_model::AVAILABLE_MODELS, messages::MESSAGES};

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
    true
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
    } else {
        //if not found offer to walk the user through a guided set up and then save to json, and use it

        println!("Config file not found. Using default values.");
        ConfigTC::default() // Return default if file not found
    }
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
