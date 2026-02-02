//! Error types for the Claude API client.

use thiserror::Error;

/// Errors that can occur when using the Claude client.
#[derive(Debug, Error)]
pub enum Error {
    #[error("API key not configured")]
    NoApiKey,

    #[error("Network error: {0}")]
    Network(String),

    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },

    #[error("Failed to parse response: {0}")]
    Parse(String),

    #[error("Invalid configuration: {0}")]
    Config(String),
}
