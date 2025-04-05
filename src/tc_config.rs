use crate::{
    messages::MESSAGES,
    utils::{confirm_action, sequence_equals},
};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{env, error::Error, fs::File, io::stdin, path::PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ConfigTC {
    #[serde(default = "default_streaming")]
    pub(crate) enable_streaming: bool,

    #[serde(default)]
    pub(crate) model: String,

    #[serde(default)]
    pub(crate) all_models: Vec<String>,

    #[serde(default = "default_dev_message")]
    pub(crate) dev_message: String,

    #[serde(default)]
    pub(crate) preview_md: bool,

    #[serde(default = "default_anthropic")]
    pub(crate) anthropic_enabled: bool,

    #[serde(default = "default_openai")]
    pub(crate) openai_enabled: bool,

    #[serde(default)]
    pub(crate) message_boxes_enabled: bool,
}

fn default_dev_message() -> String {
    MESSAGES["developer"].to_string()
}

fn default_streaming() -> bool {
    false
}

fn default_anthropic() -> bool {
    env::var("ANTHROPIC_API_KEY").is_ok()
}

fn default_openai() -> bool {
    env::var("OPENAI_API_KEY").is_ok()
}

pub async fn load_config() -> Result<ConfigTC, Box<dyn Error>> {
    if !default_anthropic() && !default_openai() {
        eprintln!(
            "\nNo API keys detected, you must have an Anthropic key or an OpenAI key to use this app. Support for Local AI instances in development.\n"
        );
        return Ok(ConfigTC::default(vec![]));
    }

    let all_models =
        crate::utils::get_all_model_names(default_anthropic(), default_openai()).await?;

    if let Ok(file) = File::open(get_config_path()) {
        match serde_json::from_reader::<File, ConfigTC>(file) {
            Ok(mut config) => {
                if !sequence_equals(&config.all_models, &all_models) {
                    config.all_models = all_models.clone();
                    // Update the configuration file with the new models list
                    write_config(&config)?;
                }
                if !all_models.contains(&config.model) {
                    eprintln!(
                        "\nInvalid model found in config. Using: {}",
                        all_models.first().unwrap()
                    );
                    config.model = all_models.first().unwrap().to_owned();
                }
                Ok(config)
            }
            Err(_) => {
                println!("\nFailed to load config. Using default values.");
                Ok(ConfigTC::default(all_models))
            }
        }
    } else if confirm_action("No config file found. Would you like to set one up? (y/n)") {
        let mut config = ConfigTC::default(all_models.clone());
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
                    "\nInvalid model found in config. Using: {}",
                    all_models.first().unwrap()
                );
                config.model = all_models.first().unwrap().to_owned();
            }
        }

        config.enable_streaming = confirm_action(
            "Would you like to enable streaming for eligible OpenAI models (experimental, Anthropic models under development)? (y/n)",
        );

        config.preview_md = confirm_action(
            "Would you like to display non-streamed model responses as rendered markdown (experimental)? (y/n)",
        );

        config.message_boxes_enabled = confirm_action(
            "Would you like to display chat messages in text boxes? (experimental, disables streaming and markdown) (y/n)",
        );

        if config.message_boxes_enabled {
            config.enable_streaming = false;
            config.preview_md = false;
        }

        if confirm_action("Would you like to write a custom developer message for the AI? (y/n)") {
            let mut prompt = String::new();
            stdin().read_line(&mut prompt).expect("failed to read line");
            config.dev_message = prompt;
        }

        println!(
            "\nConfiguration:\nModel: {}\nEnable Streaming: {}\nPreview Markdown: {}\nMessage Boxes: {}\nDeveloper Message:\n {}\n",
            config.model,
            config.enable_streaming,
            config.preview_md,
            config.message_boxes_enabled,
            config.dev_message
        );
        write_config(&config).expect("Error writing config");

        Ok(config)
    } else {
        println!("Using default values.");
        Ok(ConfigTC::default(all_models))
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
    pub fn default(all_models: Vec<String>) -> Self {
        Self {
            enable_streaming: default_streaming(),
            model: all_models.first().unwrap().to_owned(),
            all_models,
            dev_message: default_dev_message(),
            preview_md: true,
            anthropic_enabled: default_anthropic(),
            openai_enabled: default_openai(),
            message_boxes_enabled: false,
        }
    }
}
