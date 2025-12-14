// Only expose what's needed for tests
pub use conversation::{
    Provider, ConversationContext, Message,
    OpenAIRequest, ResponsesRequest, AnthropicRequest,
};

pub use utils::calculate_message_width;

pub(crate) mod chat_client;
pub(crate) mod commands;
pub(crate) mod conversation;
pub(crate) mod message_printer;
pub(crate) mod messages;
pub(crate) mod preview_md;
pub(crate) mod spinner;
pub(crate) mod tc_config;
pub(crate) mod utils;
