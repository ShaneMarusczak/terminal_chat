// Expose modules publicly for tests and external use
pub mod chat_client;
pub mod commands;
pub mod conversation;
pub mod message_printer;
pub mod messages;
pub mod preview_md;
pub mod spinner;
pub mod tc_config;
pub mod utils;

// Re-export commonly used types for convenience
pub use conversation::{
    Provider, ConversationContext, Message,
    OpenAIRequest, ResponsesRequest, AnthropicRequest,
};

pub use utils::calculate_message_width;
