#![allow(clippy::panic)]

use tc::*;

// ============================================================================
// Utility Tests
// ============================================================================

#[test]
fn test_calculate_message_width() {
    let short_text = "Hello";
    let (width, _) = calculate_message_width(short_text, 70, 80);
    assert!(width >= 6); // At least minimum width

    let long_text = "This is a much longer message that should use the calculated width based on terminal size and percentage";
    let (width, _) = calculate_message_width(long_text, 70, 80);
    assert!(width > 10);

    // Multi-line should use different calculation
    let multiline = "Line 1\nLine 2\nLine 3";
    let (width_multi, _) = calculate_message_width(multiline, 70, 80);
    assert!(width_multi > 0);
}

// ============================================================================
// Conversation Tests
// ============================================================================

fn create_test_context() -> ConversationContext {
    let mut ctx = ConversationContext::new("gpt-4o");
    ctx.input.push(Message {
        role: "developer".to_string(),
        content: "You are a helpful assistant.".to_string(),
    });
    ctx.input.push(Message {
        role: "user".to_string(),
        content: "Hello!".to_string(),
    });
    ctx.input.push(Message {
        role: "assistant".to_string(),
        content: "Hi! How can I help?".to_string(),
    });
    ctx
}

#[test]
fn test_provider_from_model_name() {
    assert_eq!(Provider::from_model_name("claude-3-5-sonnet"), Provider::Anthropic);
    assert_eq!(Provider::from_model_name("claude-opus-4"), Provider::Anthropic);
    assert_eq!(Provider::from_model_name("gpt-4o"), Provider::OpenAI);
    assert_eq!(Provider::from_model_name("o1"), Provider::OpenAI);
    assert_eq!(Provider::from_model_name("GPT-4"), Provider::OpenAI);
    assert_eq!(Provider::from_model_name("CLAUDE-3"), Provider::Anthropic);
}

#[test]
fn test_openai_request_from_context() {
    let ctx = create_test_context();
    let request = OpenAIRequest::from_context(&ctx);

    assert_eq!(request.model, "gpt-4o");
    // Should filter out developer messages
    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.messages[0].role, "user");
    assert_eq!(request.messages[0].content, "Hello!");
    assert_eq!(request.messages[1].role, "assistant");
}

#[test]
fn test_responses_request_from_context() {
    let ctx = create_test_context();
    let request = ResponsesRequest::from_context(&ctx);

    assert_eq!(request.model, "gpt-4o");
    // Should include all messages including developer
    assert_eq!(request.input.len(), 3);
    assert_eq!(request.input[0].role, "developer");
    assert_eq!(request.input[1].role, "user");
    assert_eq!(request.input[2].role, "assistant");
}

#[test]
fn test_anthropic_request_from_context() {
    let ctx = create_test_context();
    let request = AnthropicRequest::from_context(&ctx, 4096);

    assert_eq!(request.model, "gpt-4o");
    assert_eq!(request.max_tokens, 4096);
    assert_eq!(request.system, "You are a helpful assistant.");
    // Should filter out developer messages
    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.messages[0].role, "user");
    assert_eq!(request.messages[1].role, "assistant");
}

#[test]
fn test_conversation_context_new() {
    let ctx = ConversationContext::new("claude-3-5-sonnet");
    assert_eq!(ctx.model, "claude-3-5-sonnet");
    assert!(ctx.input.is_empty());
    assert!(ctx.yank_target.is_none());
}

#[test]
fn test_yank_target_persistence() {
    let mut ctx = ConversationContext::new("gpt-4o");
    assert!(ctx.yank_target.is_none());

    ctx.yank_target = Some("Test content".to_string());
    assert_eq!(ctx.yank_target, Some("Test content".to_string()));
}

#[test]
fn test_openai_request_serialization() {
    let ctx = create_test_context();
    let request = OpenAIRequest::from_context(&ctx);

    let json = match serde_json::to_string(&request) {
        Ok(json) => json,
        Err(e) => panic!("Failed to serialize OpenAIRequest: {}", e),
    };
    // Should use "messages" key
    assert!(json.contains("\"messages\""));
    assert!(!json.contains("\"input\""));
}

#[test]
fn test_responses_request_serialization() {
    let ctx = create_test_context();
    let request = ResponsesRequest::from_context(&ctx);

    let json = match serde_json::to_string(&request) {
        Ok(json) => json,
        Err(e) => panic!("Failed to serialize ResponsesRequest: {}", e),
    };
    // Should use "input" key
    assert!(json.contains("\"input\""));
    assert!(!json.contains("\"messages\""));
}
