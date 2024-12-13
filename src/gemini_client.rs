use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::{Client as ReqwestClient, RequestBuilder, StatusCode};
use serde_json::{json, Value};

use crate::{
    types::{
        GeminiCandidate, GeminiContent, GeminiError, GeminiFunctionDeclaration,
        GeminiGenerationConfig, GeminiPart, GeminiRequest, GeminiResponse, GeminiSafetySetting,
        GeminiTool,
    },
    LLMClient, LLMConfig,
};

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";

#[derive(Debug, Clone)]
pub struct GeminiClient {
    client: ReqwestClient,
    config: LLMConfig,
    safety_settings: Option<Vec<GeminiSafetySetting>>,
    tools: Vec<GeminiTool>,
}

impl GeminiClient {
    fn build_request(&self, content: &str) -> Result<(RequestBuilder, GeminiRequest)> {
        let generation_config = GeminiGenerationConfig {
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            top_k: self.config.top_k,
            max_output_tokens: self.config.max_tokens,
            stop_sequences: self.config.stop_sequences.clone(),
        };

        let contents = vec![GeminiContent {
            parts: vec![GeminiPart::Text {
                text: content.to_string(),
            }],
            role: Some("user".to_string()),
        }];

        let request = GeminiRequest {
            contents,
            tools: self.tools.clone(),
            safety_settings: self.safety_settings.clone(),
            generation_config: Some(generation_config),
        };

        let url = if self.config.streaming {
            format!(
                "{}/{}:streamGenerateContent?key={}",
                GEMINI_API_BASE, self.config.model, self.config.api_key
            )
        } else {
            format!(
                "{}/{}:generateContent?key={}",
                GEMINI_API_BASE, self.config.model, self.config.api_key
            )
        };

        let request_builder = self.client.post(&url).json(&request);

        Ok((request_builder, request))
    }

    fn extract_text_from_candidate(candidate: &GeminiCandidate) -> Option<String> {
        for part in &candidate.content.parts {
            if let GeminiPart::Text { text } = part {
                return Some(text.clone());
            }
        }
        None
    }

    // Additional configuration methods
    pub fn with_safety_settings(mut self, safety_settings: Vec<GeminiSafetySetting>) -> Self {
        self.safety_settings = Some(safety_settings);
        self
    }

    pub fn with_tools(mut self, tools: Vec<GeminiTool>) -> Self {
        self.tools = tools;
        self
    }

    pub fn function_declaration(
        name: &str,
        description: &str,
        parameters: Value,
    ) -> GeminiFunctionDeclaration {
        GeminiFunctionDeclaration {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
        }
    }
}

#[async_trait]
impl LLMClient for GeminiClient {
    async fn send_message(&self, content: &str) -> Result<String> {
        let (request_builder, _) = self.build_request(content)?;
        let response = request_builder
            .send()
            .await
            .context("Failed to send request")?;

        match response.status() {
            StatusCode::OK => {
                let gemini_response: GeminiResponse = response.json().await?;
                if let Some(candidate) = gemini_response.candidates.first() {
                    if let Some(text) = Self::extract_text_from_candidate(candidate) {
                        Ok(text)
                    } else {
                        Err(anyhow!("No text content in response"))
                    }
                } else {
                    Err(anyhow!("No candidates in response"))
                }
            }
            _ => {
                let error_text = response.text().await?;
                let error: GeminiError =
                    serde_json::from_str(&error_text).context("Failed to parse error response")?;
                Err(anyhow!(
                    "API Error ({}): {}",
                    error.error.code,
                    error.error.message
                ))
            }
        }
    }

    async fn stream_message<F, Fut>(&self, content: &str, mut callback: F) -> Result<()>
    where
        F: FnMut(String) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let (request_builder, _) = self.build_request(content)?;
        let response = request_builder
            .send()
            .await
            .context("Failed to send request")?;

        match response.status() {
            StatusCode::OK => {
                let mut response = response;
                while let Some(chunk) = response.chunk().await? {
                    let chunk_str = String::from_utf8(chunk.to_vec())
                        .context("Failed to decode chunk as UTF-8")?;

                    if chunk_str.trim().is_empty() {
                        continue;
                    }

                    let chunk_response: GeminiResponse = serde_json::from_str(&chunk_str)
                        .context("Failed to parse chunk as GeminiResponse")?;

                    for candidate in chunk_response.candidates {
                        for part in candidate.content.parts {
                            match part {
                                GeminiPart::Text { text } => {
                                    callback(text).await;
                                }
                                GeminiPart::FunctionCall { function_call } => {
                                    callback(serde_json::to_string(&function_call)?).await;
                                }
                                GeminiPart::FunctionResponse { .. } => {}
                            }
                        }
                    }
                }
                Ok(())
            }
            _ => {
                let error_text = response.text().await?;
                let error: GeminiError =
                    serde_json::from_str(&error_text).context("Failed to parse error response")?;
                Err(anyhow!(
                    "API Error ({}): {}",
                    error.error.code,
                    error.error.message
                ))
            }
        }
    }

    async fn send_message_raw(&self, content: &str) -> Result<Value> {
        let (request_builder, _) = self.build_request(content)?;
        let response = request_builder
            .send()
            .await
            .context("Failed to send request")?;

        match response.status() {
            StatusCode::OK => {
                let json_value = response.json().await?;
                Ok(json_value)
            }
            _ => {
                let error_text = response.text().await?;
                let error: GeminiError =
                    serde_json::from_str(&error_text).context("Failed to parse error response")?;
                Err(anyhow!(
                    "API Error ({}): {}",
                    error.error.code,
                    error.error.message
                ))
            }
        }
    }

    fn with_config(config: LLMConfig) -> Result<Self> {
        Ok(Self {
            client: ReqwestClient::new(),
            config,
            safety_settings: None,
            tools: Vec::new(),
        })
    }

    fn update_config(&mut self, config: LLMConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }
}

/// Converts an OpenAPI-style function schema to a GeminiFunctionDeclaration.
/// This function takes a schema that follows the OpenAPI format (with input_schema)
/// and converts it to the format expected by Gemini's function declarations.
pub fn convert_to_function_declaration(schema: &Value) -> GeminiFunctionDeclaration {
    // Extract the basic function information from the schema
    let name = schema
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or_default();

    let description = schema
        .get("description")
        .and_then(|d| d.as_str())
        .unwrap_or_default();

    // Get the input schema object which contains our properties
    let input_schema = schema
        .get("input_schema")
        .and_then(|s| s.as_object())
        .expect("input_schema must be a valid object");

    // Extract properties and convert them to Gemini's expected format
    let properties = input_schema
        .get("properties")
        .and_then(|p| p.as_object())
        .expect("properties must be a valid object");

    // Create a new map and insert converted properties
    let mut converted_properties = serde_json::Map::new();

    // Iterate over properties and convert each one
    for (key, value) in properties {
        let prop_obj = value.as_object().unwrap();
        converted_properties.insert(
            key.to_string(), // Convert &String to String by cloning
            json!({
                "type": prop_obj.get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("string")
                    .to_uppercase(),
                "description": prop_obj.get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
            }),
        );
    }

    // Extract required fields
    let required = input_schema
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string()) // Convert &str to String
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // Create the final schema in Gemini's format
    let gemini_schema = json!({
        "type": "OBJECT",
        "properties": converted_properties,
        "required": required
    });

    // Use GeminiClient's function_declaration to create the final declaration
    GeminiClient::function_declaration(name, description, gemini_schema)
}
