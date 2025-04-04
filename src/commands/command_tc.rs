use super::command_context::CommandContext;
use std::future::Future;
use std::pin::Pin;

pub type CommandResult = Result<(), Box<dyn std::error::Error>>;
pub type RunFunc<'a> =
    fn(Option<CommandContext<'a>>) -> Pin<Box<dyn Future<Output = CommandResult>>>;

pub struct CommandTC<'a> {
    pub name: &'static str,
    pub description: &'static str,
    pub run: RunFunc<'a>,
}
