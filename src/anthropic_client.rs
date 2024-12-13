use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::{Client as ReqwestClient, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{AnthropicChatCompletionChunk, AnthropicErrorMessage, LLMClient, LLMConfig};

#[derive(Debug, Deserialize)]
pub struct AnthropicResponse {
    pub id: String,
    pub model: String,
    pub stop_reason: String,
    pub role: String,
    pub content: Vec<ContentItem>,
    pub usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ContentItem {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
}

#[derive(Debug, Clone)]
pub struct AnthropicClient {
    client: ReqwestClient,
    config: LLMConfig,
    version: String,
    beta: Option<String>,
    verbose: bool,
    metadata: Option<Value>,
}

impl AnthropicClient {
    fn build_request(&self, content: &str) -> Result<(RequestBuilder, HashMap<&str, Value>)> {
        let mut body_map: HashMap<&str, Value> = HashMap::new();

        // Add required fields
        body_map.insert("model", json!(self.config.model));
        body_map.insert("messages", json!([{"role": "user", "content": content}]));

        // Add optional fields from config
        if let Some(max_tokens) = self.config.max_tokens {
            body_map.insert("max_tokens", json!(max_tokens));
        }

        if let Some(temperature) = self.config.temperature {
            body_map.insert("temperature", json!(temperature));
        }

        if let Some(system) = &self.config.system_prompt {
            body_map.insert(
                "system",
                json!([{
                    "type": "text",
                    "text": system,
                    "cache_control": {"type": "ephemeral"}
                }]),
            );
        }

        if let Some(tools) = &self.config.tools {
            body_map.insert("tools", tools.clone());
        }

        if let Some(stop_sequences) = &self.config.stop_sequences {
            body_map.insert("stop_sequences", json!(stop_sequences));
        }

        if let Some(top_p) = self.config.top_p {
            body_map.insert("top_p", json!(top_p));
        }

        if let Some(top_k) = self.config.top_k {
            body_map.insert("top_k", json!(top_k));
        }

        if let Some(metadata) = &self.metadata {
            body_map.insert("metadata", metadata.clone());
        }

        let mut request_builder = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", &self.version)
            .header("content-type", "application/json");

        if let Some(beta) = &self.beta {
            request_builder = request_builder.header("anthropic-beta", beta);
        }

        Ok((request_builder, body_map))
    }

    // Additional configuration methods
    pub fn with_beta(mut self, beta: &str) -> Self {
        self.beta = Some(beta.to_owned());
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[async_trait]
impl LLMClient for AnthropicClient {
    async fn send_message(&self, content: &str) -> Result<String> {
        let (request_builder, body_map) = self.build_request(content)?;
        let response = request_builder
            .json(&body_map)
            .send()
            .await
            .context("Failed to send request")?;

        match response.status() {
            StatusCode::OK => {
                let anthropic_response: AnthropicResponse = response.json().await?;
                if let Some(ContentItem::Text { text }) = anthropic_response.content.first() {
                    Ok(text.clone())
                } else {
                    Err(anyhow!("No text content in response"))
                }
            }
            status => {
                let error_text = response.text().await?;
                Err(anyhow!("Request failed ({}): {}", status, error_text))
            }
        }
    }

    async fn stream_message<F, Fut>(&self, content: &str, mut callback: F) -> Result<()>
    where
        F: FnMut(String) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let mut body_map = self.build_request(content)?.1;
        body_map.insert("stream", json!(true));

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", &self.version)
            .header("content-type", "application/json")
            .json(&body_map)
            .send()
            .await
            .context("Failed to send request")?;

        match response.status() {
            StatusCode::OK => {
                let mut buffer = String::new();
                let mut response = response;

                while let Some(chunk) = response.chunk().await? {
                    let s = std::str::from_utf8(&chunk)?;
                    buffer.push_str(s);

                    loop {
                        if let Some(index) = buffer.find("\n\n") {
                            let chunk = buffer[..index].to_string();
                            buffer.drain(..=index + 1);

                            if self.verbose {
                                callback(chunk.clone()).await;
                            } else {
                                if chunk == "data: [DONE]" {
                                    break;
                                }

                                let processed_chunk = self.process_stream_chunk(&chunk)?;
                                if !processed_chunk.is_empty() {
                                    callback(processed_chunk).await;
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }
                Ok(())
            }
            status => {
                let error_text = response.text().await?;
                Err(anyhow!(
                    "Stream request failed ({}): {}",
                    status,
                    error_text
                ))
            }
        }
    }

    async fn send_message_raw(&self, content: &str) -> Result<Value> {
        let (request_builder, body_map) = self.build_request(content)?;
        let response = request_builder
            .json(&body_map)
            .send()
            .await
            .context("Failed to send request")?;

        match response.status() {
            StatusCode::OK => {
                let json_value = response.json().await?;
                Ok(json_value)
            }
            status => {
                let error_text = response.text().await?;
                Err(anyhow!("Request failed ({}): {}", status, error_text))
            }
        }
    }

    fn with_config(config: LLMConfig) -> Result<Self> {
        Ok(Self {
            client: ReqwestClient::new(),
            config,
            version: "2023-06-01".to_string(),
            beta: None,
            verbose: false,
            metadata: None,
        })
    }

    fn update_config(&mut self, config: LLMConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }
}

// Helper methods implementation
impl AnthropicClient {
    fn process_stream_chunk(&self, chunk: &str) -> Result<String> {
        let processed_chunk = chunk
            .trim_start_matches("event: message_start")
            .trim_start_matches("event: content_block_start")
            .trim_start_matches("event: ping")
            .trim_start_matches("event: content_block_delta")
            .trim_start_matches("event: content_block_stop")
            .trim_start_matches("event: message_delta")
            .trim_start_matches("event: message_stop")
            .to_string();

        let cleaned_string = processed_chunk
            .trim_start()
            .strip_prefix("data: ")
            .unwrap_or(&processed_chunk);

        match serde_json::from_str::<AnthropicChatCompletionChunk>(cleaned_string) {
            Ok(d) => {
                if let Some(delta) = d.delta {
                    if let Some(content) = delta.text {
                        return Ok(content);
                    }
                }
                Ok(String::new())
            }
            Err(_) => {
                // Try parsing as error message
                let processed_chunk = cleaned_string
                    .trim_start_matches("event: error")
                    .to_string();
                let cleaned_string = processed_chunk
                    .trim_start()
                    .strip_prefix("data: ")
                    .unwrap_or(&processed_chunk);

                if let Ok(error_message) =
                    serde_json::from_str::<AnthropicErrorMessage>(cleaned_string)
                {
                    return Err(anyhow!(
                        "{}: {}",
                        error_message.error.error_type,
                        error_message.error.message
                    ));
                }

                eprintln!(
                    "Couldn't parse AnthropicChatCompletionChunk or AnthropicErrorMessage: {}",
                    cleaned_string
                );
                Ok(String::new())
            }
        }
    }
}
