//! Minimal Anthropic Claude API client.
//!
//! This crate provides a focused client for Claude's Messages API with:
//! - Non-streaming and streaming completions
//! - Tool use support
//! - Proper SSE parsing for streaming responses

mod api_types;
mod client;
mod error;
mod streaming;
mod types;

pub use client::Claude;
pub use error::Error;
pub use types::{
    ContentBlock, Message, Request, Response, Role, StopReason, StreamEvent, Tool, ToolChoice,
    ToolResult, ToolUse, Usage,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::DEFAULT_MODEL;

    #[test]
    fn test_client_creation() {
        let client = Claude::new("test-key");
        assert_eq!(client.model, DEFAULT_MODEL);
    }

    #[test]
    fn test_client_with_model() {
        let client = Claude::new("test-key").with_model("claude-3-opus");
        assert_eq!(client.model, "claude-3-opus");
    }

    #[test]
    fn test_request_builder() {
        let request = Request::new(vec![Message::user("Hello")])
            .with_system("You are a helpful assistant")
            .with_max_tokens(1000)
            .with_temperature(0.7);

        assert_eq!(request.max_tokens, 1000);
        assert!(request.system.is_some());
        assert_eq!(request.temperature, Some(0.7));
    }

    #[test]
    fn test_message_creation() {
        let user_msg = Message::user("Hello");
        assert!(matches!(user_msg.role, Role::User));
        assert_eq!(user_msg.content.len(), 1);

        let assistant_msg = Message::assistant("Hi there");
        assert!(matches!(assistant_msg.role, Role::Assistant));
    }

    #[test]
    fn test_tool_result() {
        let success = ToolResult::success("worked");
        assert!(!success.is_error);
        assert_eq!(success.content, "worked");

        let error = ToolResult::error("failed");
        assert!(error.is_error);
        assert_eq!(error.content, "failed");
    }
}
