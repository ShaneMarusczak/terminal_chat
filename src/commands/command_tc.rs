use std::future::Future;
use std::pin::Pin;

use super::CommandResult;
use super::command_context::CommandContext;

pub struct CommandTC {
    pub name: &'static str,
    pub description: &'static str,
    pub run: fn(CommandContext) -> Pin<Box<dyn Future<Output = CommandResult>>>,
}
