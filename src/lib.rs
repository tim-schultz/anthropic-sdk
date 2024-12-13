use anyhow::Result;

// Module declarations
mod anthropic_client;
mod gemini_client;
mod llm_client;
mod traits;
mod types;

// Re-export the Anthropic client
pub use anthropic_client::{AnthropicClient, AnthropicResponse, ContentItem};

// Re-export Gemini client
pub use crate::gemini_client::GeminiClient;

// Re-export common trait
pub use crate::traits::{LLMClient, LLMConfig};

pub use crate::llm_client::{ClientType, LLMClientType};

// Re-export types
pub use crate::types::{
    // Anthropic types
    AnthropicChatCompletionChunk,
    AnthropicErrorMessage,
    // Gemini types
    GeminiCandidate,
    GeminiContent,
    GeminiError,
    GeminiErrorDetails,
    GeminiFunctionCall,
    GeminiFunctionDeclaration,
    GeminiFunctionResponse,
    GeminiGenerationConfig,
    GeminiPart,
    GeminiRequest,
    GeminiResponse,
    GeminiSafetySetting,
    GeminiTool,
    GeminiUsage,
};
