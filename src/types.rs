use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct AnthropicUsage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnthropicContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnthropicTextDelta {
    #[serde(rename = "type")]
    pub delta_type: Option<String>,
    pub text: Option<String>,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: Option<AnthropicUsage>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnthropicMessage {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub message_type: String,
    pub role: Option<String>,
    pub content: Option<Vec<AnthropicContentBlock>>,
    pub model: Option<String>,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: Option<AnthropicUsage>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub struct AnthropicChatCompletionChunk {
    #[serde(rename = "type")]
    pub event_type: String,
    pub index: Option<usize>,
    pub delta: Option<AnthropicTextDelta>,
    pub message: Option<AnthropicMessage>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnthropicErrorMessage {
    #[serde(rename = "type")]
    pub error_type: String,
    pub error: AnthropicErrorDetails,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnthropicErrorDetails {
    pub details: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

// Gemini API Types
#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiContent {
    pub parts: Vec<GeminiPart>,
    pub role: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum GeminiPart {
    Text {
        text: String,
    },
    FunctionCall {
        function_call: GeminiFunctionCall,
    },
    FunctionResponse {
        function_response: GeminiFunctionResponse,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiFunctionCall {
    pub name: String,
    pub args: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiFunctionResponse {
    pub name: String,
    pub response: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiRequest {
    pub contents: Vec<GeminiContent>,
    pub tools: Vec<GeminiTool>,
    pub safety_settings: Option<Vec<GeminiSafetySetting>>,
    pub generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeminiTool {
    pub function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeminiFunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeminiSafetySetting {
    pub category: String,
    pub threshold: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiGenerationConfig {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
    pub max_output_tokens: Option<i32>,
    pub stop_sequences: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiResponse {
    pub candidates: Vec<GeminiCandidate>,
    pub usage_metadata: Option<GeminiUsage>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiCandidate {
    pub content: GeminiContent,
    pub finish_reason: Option<String>,
    pub safety_ratings: Option<Vec<GeminiSafetyRating>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiSafetyRating {
    pub category: String,
    pub probability: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiUsage {
    pub prompt_token_count: i32,
    pub candidates_token_count: i32,
    pub total_token_count: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiError {
    pub error: GeminiErrorDetails,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiErrorDetails {
    pub code: i32,
    pub message: String,
    pub status: String,
}
