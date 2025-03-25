pub mod change_model;
pub mod clear;
pub mod command_context;
pub mod command_tc;
pub mod commands_registry;
pub mod debug;
pub mod document;
pub mod gf;
pub mod handle_commands;
pub mod help;
pub mod image;
pub mod quit;
pub mod readme;
pub mod sh;

pub type CommandResult = Result<(), Box<dyn std::error::Error>>;
