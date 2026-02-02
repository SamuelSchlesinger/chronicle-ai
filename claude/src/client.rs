//! Claude API client implementation.

use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use std::pin::Pin;
use tokio_stream::Stream;

use crate::api_types::{ApiContent, ApiMessage, ApiRequest, ApiResponse, ApiTool, ApiToolChoice};
use crate::error::Error;
use crate::streaming::parse_sse_events_buffered;
use crate::types::{
    ContentBlock, Message, Request, Response, Role, StopReason, StreamEvent, ToolChoice,
    ToolResult, ToolUse, Usage,
};

const API_BASE: &str = "https://api.anthropic.com/v1";
const API_VERSION: &str = "2023-06-01";
pub(crate) const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

/// Claude API client for making requests to Anthropic's Claude models.
///
/// Handles authentication, request serialization, and response parsing.
/// Supports both synchronous completions and streaming responses.
///
/// # Example
///
/// ```no_run
/// use claude::{Claude, Request, Message};
///
/// # async fn example() -> Result<(), claude::Error> {
/// let client = Claude::from_env()?;
/// let response = client.complete(
///     Request::new(vec![Message::user("What is the capital of France?")])
///         .with_system("You are a helpful assistant.")
/// ).await?;
/// println!("{}", response.text());
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Claude {
    client: reqwest::Client,
    api_key: String,
    pub(crate) model: String,
}

impl Claude {
    /// Creates a new Claude client with the provided API key.
    ///
    /// Initializes with the default model and HTTP timeouts (120s total, 30s connect).
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your Anthropic API key from <https://console.anthropic.com/>
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .connect_timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            api_key: api_key.into(),
            model: DEFAULT_MODEL.to_string(),
        }
    }

    /// Creates a Claude client using the `ANTHROPIC_API_KEY` environment variable.
    ///
    /// This is the recommended way to initialize the client, keeping API keys out of source code.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoApiKey`] if `ANTHROPIC_API_KEY` is not set.
    pub fn from_env() -> Result<Self, Error> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| Error::NoApiKey)?;
        Ok(Self::new(api_key))
    }

    /// Sets the default model for this client.
    ///
    /// Can be overridden per-request using [`Request::with_model`].
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Sends a completion request and returns the full response.
    ///
    /// This is the primary method for non-streaming interactions with Claude.
    /// Waits for the complete response before returning.
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails or the API returns an error.
    pub async fn complete(&self, request: Request) -> Result<Response, Error> {
        let api_request = self.build_api_request(&request, false);
        let headers = self.build_headers()?;

        let response = self
            .client
            .post(format!("{API_BASE}/messages"))
            .headers(headers)
            .json(&api_request)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status,
                message: body,
            });
        }

        let api_response: ApiResponse = response
            .json()
            .await
            .map_err(|e| Error::Parse(e.to_string()))?;

        Ok(self.parse_response(api_response))
    }

    /// Sends a completion request and returns a stream of response events.
    ///
    /// Use for real-time streaming, which provides better UX for longer responses.
    /// Events include text deltas, tool use, and message lifecycle events.
    pub async fn stream(
        &self,
        request: Request,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, Error>> + Send>>, Error> {
        let api_request = self.build_api_request(&request, true);
        let headers = self.build_headers()?;

        let response = self
            .client
            .post(format!("{API_BASE}/messages"))
            .headers(headers)
            .json(&api_request)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status,
                message: body,
            });
        }

        // Use scan to maintain a buffer for incomplete SSE events across chunks
        let stream = response
            .bytes_stream()
            .scan(String::new(), |buffer, result| {
                let events = match result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        parse_sse_events_buffered(buffer)
                    }
                    Err(e) => vec![Err(Error::Network(e.to_string()))],
                };
                // Return Some to continue, the events vector
                futures::future::ready(Some(events))
            })
            .flat_map(futures::stream::iter);

        Ok(Box::pin(stream))
    }

    /// Run a tool use loop until completion.
    ///
    /// Given a request with tools and an executor function, this method will:
    /// 1. Send the request to Claude
    /// 2. If Claude calls tools, execute them using the provided function
    /// 3. Send tool results back and repeat until Claude stops using tools
    pub async fn complete_with_tools<F, Fut>(
        &self,
        mut request: Request,
        mut executor: F,
    ) -> Result<Response, Error>
    where
        F: FnMut(ToolUse) -> Fut,
        Fut: std::future::Future<Output = ToolResult>,
    {
        loop {
            let response = self.complete(request.clone()).await?;

            if response.stop_reason != StopReason::ToolUse {
                return Ok(response);
            }

            // Collect tool uses
            let tool_uses: Vec<ToolUse> = response
                .content
                .iter()
                .filter_map(|block| {
                    if let ContentBlock::ToolUse { id, name, input } = block {
                        Some(ToolUse {
                            id: id.clone(),
                            name: name.clone(),
                            input: input.clone(),
                        })
                    } else {
                        None
                    }
                })
                .collect();

            if tool_uses.is_empty() {
                return Ok(response);
            }

            // Add assistant response to messages
            request.messages.push(Message {
                role: Role::Assistant,
                content: response.content.clone(),
            });

            // Execute tools and collect results
            let mut tool_results = Vec::new();
            for tool_use in tool_uses {
                let result = executor(tool_use.clone()).await;
                tool_results.push(ContentBlock::ToolResult {
                    tool_use_id: tool_use.id,
                    content: result.content,
                    is_error: result.is_error,
                });
            }

            // Add tool results as user message
            request.messages.push(Message {
                role: Role::User,
                content: tool_results,
            });
        }
    }

    fn build_headers(&self) -> Result<HeaderMap, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.api_key)
                .map_err(|e| Error::Config(format!("Invalid API key: {e}")))?,
        );
        headers.insert("anthropic-version", HeaderValue::from_static(API_VERSION));
        Ok(headers)
    }

    fn build_api_request(&self, request: &Request, stream: bool) -> ApiRequest {
        let messages: Vec<ApiMessage> = request
            .messages
            .iter()
            .map(|m| ApiMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: m.content.iter().map(|c| c.into()).collect(),
            })
            .collect();

        let tools: Option<Vec<ApiTool>> = request.tools.as_ref().map(|tools| {
            tools
                .iter()
                .map(|t| ApiTool {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    input_schema: t.input_schema.clone(),
                })
                .collect()
        });

        ApiRequest {
            model: request.model.clone().unwrap_or_else(|| self.model.clone()),
            max_tokens: request.max_tokens,
            system: request.system.clone(),
            messages,
            temperature: request.temperature,
            tools,
            tool_choice: request.tool_choice.as_ref().map(|tc| match tc {
                ToolChoice::Auto => ApiToolChoice {
                    r#type: "auto".to_string(),
                    name: None,
                },
                ToolChoice::Any => ApiToolChoice {
                    r#type: "any".to_string(),
                    name: None,
                },
                ToolChoice::Tool { name } => ApiToolChoice {
                    r#type: "tool".to_string(),
                    name: Some(name.clone()),
                },
            }),
            stream,
        }
    }

    fn parse_response(&self, api_response: ApiResponse) -> Response {
        let content: Vec<ContentBlock> = api_response
            .content
            .into_iter()
            .map(|c| match c {
                ApiContent::Text { text } => ContentBlock::Text { text },
                ApiContent::ToolUse { id, name, input } => {
                    ContentBlock::ToolUse { id, name, input }
                }
                ApiContent::Thinking { thinking } => ContentBlock::Thinking { thinking },
            })
            .collect();

        let stop_reason = match api_response.stop_reason.as_str() {
            "end_turn" => StopReason::EndTurn,
            "max_tokens" => StopReason::MaxTokens,
            "stop_sequence" => StopReason::StopSequence,
            "tool_use" => StopReason::ToolUse,
            _ => StopReason::EndTurn,
        };

        Response {
            id: api_response.id,
            model: api_response.model,
            content,
            stop_reason,
            usage: Usage {
                input_tokens: api_response.usage.input_tokens,
                output_tokens: api_response.usage.output_tokens,
            },
        }
    }
}
