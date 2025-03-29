use std::{collections::HashMap, sync::LazyLock};

use crate::commands::{
    change_model::change_model_command, clear::clear_command, command_tc::CommandTC,
    debug::debug_command, document::document_command, gf::gf_command, help::help_command,
    image::image_command, load_conversation::lc_command, quit::quit_command,
    readme::readme_command, save_conversation::sc_command, sh,
};

macro_rules! register_command {
    ($registry:expr, $name:expr, $desc:expr, $func:path) => {{
        $registry.insert(
            $name,
            CommandTC {
                name: $name,
                description: $desc,
                run: |cc| Box::pin(async move { $func(cc).await }),
            },
        );
    }};
}

pub static TC_COMMANDS: LazyLock<HashMap<&str, CommandTC>> = LazyLock::new(|| {
    let mut registry = HashMap::new();

    register_command!(
        registry,
        "lc",
        "Loads a conversation from the conversations directory.",
        lc_command
    );

    register_command!(
        registry,
        "sc",
        "Saves the current conversation as JSON.",
        sc_command
    );

    register_command!(
        registry,
        "clear",
        "Clears the conversation context.",
        clear_command
    );
    register_command!(
        registry,
        "debug",
        "Prints debug information.",
        debug_command
    );
    register_command!(
        registry,
        "cm",
        "Changes the chat model.",
        change_model_command
    );
    register_command!(
        registry,
        "help",
        "Displays this help message.",
        help_command
    );
    register_command!(
        registry,
        "gf",
        "Adds file contents to the context.",
        gf_command
    );
    register_command!(
        registry,
        "readme",
        "Generates a README file.",
        readme_command
    );
    register_command!(
        registry,
        "doc",
        "Generates documentation.",
        document_command
    );
    register_command!(
        registry,
        "quit",
        "Quits this program. Also 'q'.",
        quit_command
    );
    register_command!(
        registry,
        "image",
        "Generates an image and returns its URL.",
        image_command
    );

    register_command!(
        registry,
        "sh",
        "Executes a program with arguments. Usage: sh <program> [args...]",
        sh::sh
    );
    registry
});
