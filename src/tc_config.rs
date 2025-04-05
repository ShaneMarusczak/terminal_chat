use crate::{
    messages::MESSAGES,
    utils::{confirm_action, read_user_input, sequence_equals},
};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{env, error::Error, fs::File, path::PathBuf, sync::LazyLock};

use crossterm::style::{Color, Stylize};
use std::sync::RwLock;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ConfigTC {
    #[serde(default)]
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

    #[serde(default = "default_theme")]
    pub(crate) theme: Theme,
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
        eprintln!(
            "\nNo API keys detected. You must have an Anthropic and/or an OpenAI key to use this app.\n"
        );
        return Ok(ConfigTC::default(vec![]));
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
                    eprintln!(
                        "\nInvalid model found in config. Using: {}",
                        all_models.first().unwrap()
                    );
                    config.model = all_models.first().unwrap().to_owned();
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
        write_config(&config, true).expect("Error writing config");
        config
    } else {
        println!("Using default values.");
        ConfigTC::default(all_models)
    };

    let mut global = GLOBAL_CONFIG.write().unwrap();
    *global = rv.clone();
    Ok(rv)
}

pub fn get_config() -> ConfigTC {
    GLOBAL_CONFIG.read().unwrap().clone()
}

pub fn write_config(config: &ConfigTC, prompt: bool) -> std::io::Result<()> {
    let path = get_config_path();
    if !prompt || confirm_action(&format!("Save to {}?", path.to_str().unwrap())) {
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
        let default_model = all_models
            .first()
            .unwrap_or(&"default_model_name".to_string())
            .to_owned();
        Self {
            enable_streaming: false,
            model: default_model,
            all_models,
            dev_message: default_dev_message(),
            preview_md: false,
            anthropic_enabled: default_anthropic(),
            openai_enabled: default_openai(),
            message_boxes_enabled: false,
            theme: default_theme(),
        }
    }
}

pub fn config_interview(config: &mut ConfigTC) {
    println!("\nAvailable models:");
    for (i, model) in config.all_models.iter().enumerate() {
        println!("{}) {}", i + 1, model);
    }

    config.model = loop {
        let input = read_user_input("Please select a model by typing its number:");
        if let Ok(num) = input.trim().parse::<usize>() {
            if num > 0 && num <= config.all_models.len() {
                break config.all_models[num - 1].clone();
            }
        }
        eprintln!("\nInvalid model selection. Please try again.");
    };

    config.enable_streaming = confirm_action("Enable streaming for eligible models? (y/n)");

    config.preview_md =
        confirm_action("Display non-streamed responses as rendered markdown? (y/n)");

    config.message_boxes_enabled = confirm_action(
        "Display chat messages in text boxes? (disables streaming and markdown) (y/n)",
    );

    if config.message_boxes_enabled {
        config.enable_streaming = false;
        config.preview_md = false;
    }

    if confirm_action("Write a custom developer message for the AI? (y/n)") {
        config.dev_message = read_user_input("Enter your custom message:");
    }

    set_custom_theme(config);
}

fn set_custom_theme(config: &mut ConfigTC) {
    if confirm_action("Customize theme colors? (y/n)") {
        println!("Available colors:");
        for color in available_colors() {
            println!("{}", color.with(parse_color(color)));
        }

        config.theme = Theme {
            system_color: read_valid_color("Enter system message color:"),
            user_color: read_valid_color("Enter user message color:"),
            assistant_color: read_valid_color("Enter assistant message color:"),
        };
    }
}

fn parse_color(color_name: &str) -> Color {
    match color_name.to_lowercase().as_str() {
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "black" => Color::Black,
        "dark_grey" => Color::DarkGrey,
        "light_grey" => Color::Grey, // Also known as Light Grey
        "dark_red" => Color::DarkRed,
        "dark_green" => Color::DarkGreen,
        "dark_yellow" => Color::DarkYellow,
        "dark_blue" => Color::DarkBlue,
        "dark_magenta" => Color::DarkMagenta,
        "dark_cyan" => Color::DarkCyan,
        _ => Color::Reset,
    }
}
fn read_valid_color(prompt: &str) -> String {
    loop {
        let input = read_user_input(prompt);
        if is_valid_color(&input) {
            return input;
        }
        eprintln!("Invalid color. Please try again.");
    }
}

pub(crate) fn print_config(config: &ConfigTC) {
    println!(
        "\nConfiguration:\nModel: {}\nEnable Streaming: {}\nPreview Markdown: {}\nMessage Boxes: {}\nDeveloper Message:\n {}\nTheme Colors: System: {}, User: {}, Assistant: {}",
        config.model,
        config.enable_streaming,
        config.preview_md,
        config.message_boxes_enabled,
        config.dev_message,
        config.theme.system_color,
        config.theme.user_color,
        config.theme.assistant_color
    );
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct Theme {
    #[serde(default = "default_system_color")]
    pub(crate) system_color: String,

    #[serde(default = "default_user_color")]
    pub(crate) user_color: String,

    #[serde(default = "default_assistant_color")]
    pub(crate) assistant_color: String,
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

fn is_valid_color(input: &str) -> bool {
    available_colors().contains(&input.to_lowercase().as_str())
}

fn available_colors() -> Vec<&'static str> {
    vec![
        "red",
        "green",
        "yellow",
        "blue",
        "magenta",
        "cyan",
        "white",
        "black",
        "dark_grey",
        "light_grey",
        "dark_red",
        "dark_green",
        "dark_yellow",
        "dark_blue",
        "dark_magenta",
        "dark_cyan",
    ]
}
