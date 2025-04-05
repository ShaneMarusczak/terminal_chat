use std::{collections::HashMap, sync::LazyLock};

use crate::commands::{
    change_model::change_model_command, clear::clear_command, clear_config::dc,
    command_tc::CommandTC, debug::debug_command, document::document_command,
    edit_config::ec_command, gf::gf_command, help::help_command, image::image_command,
    load_conversation::lc_command, quit::quit_command, readme::readme_command,
    save_conversation::sc_command, sh,
};

macro_rules! register_command {
    ($name:expr, $desc:expr, $func:path, $registry:expr) => {{
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
    let mut r = HashMap::new();

    register_command!("ec", "Edit the application configuration.", ec_command, r);
    register_command!("dc", "Deletes the current application config file.", dc, r);
    register_command!(
        "lc",
        "Loads a conversation from the conversations directory.",
        lc_command,
        r
    );
    register_command!(
        "sc",
        "Saves the current conversation as JSON.",
        sc_command,
        r
    );
    register_command!(
        "clear",
        "Clears the conversation context.",
        clear_command,
        r
    );
    register_command!("debug", "Prints debug information.", debug_command, r);
    register_command!("cm", "Changes the chat model.", change_model_command, r);
    register_command!("help", "Displays this help message.", help_command, r);
    register_command!("gf", "Adds file contents to the context.", gf_command, r);
    register_command!("readme", "Generates a README file.", readme_command, r);
    register_command!("doc", "Generates documentation.", document_command, r);
    register_command!("quit", "Quits this program. Also 'q'.", quit_command, r);
    register_command!(
        "image",
        "Generates an image and returns its URL.",
        image_command,
        r
    );
    register_command!(
        "sh",
        "Executes a program with arguments. Usage: sh <program> [args...]",
        sh::sh,
        r
    );

    r
});
