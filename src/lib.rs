use anyhow::{anyhow, Context, Result};
use reqwest::{Error as ReqwestError, RequestBuilder, StatusCode};
use serde::Deserialize;
use serde_json::Value;

// Module declarations
mod anthropic_client;
mod gemini_client;
mod types;

// Re-export the Anthropic client types and functionality
pub use anthropic_client::{
    AnthropicResponse, Client as AnthropicClient, ContentItem, Request as AnthropicRequest, Usage,
};

// Re-export Gemini types and client (maintained from original)
pub use crate::gemini_client::GeminiClient;
pub use crate::types::{
    GeminiCandidate, GeminiContent, GeminiError, GeminiErrorDetails, GeminiFunctionCall,
    GeminiFunctionDeclaration, GeminiFunctionResponse, GeminiGenerationConfig, GeminiPart,
    GeminiRequest, GeminiResponse, GeminiSafetySetting, GeminiTool, GeminiUsage,
};

// Re-export other types that might be needed by external crates
pub use crate::types::{AnthropicChatCompletionChunk, AnthropicErrorMessage};
