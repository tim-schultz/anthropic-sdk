use crate::{anthropic_client::AnthropicClient, gemini_client::GeminiClient, LLMClient, LLMConfig};
use anyhow::{Context, Result};
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum ClientType {
    Anthropic,
    Gemini,
}

#[derive(Debug)]
pub enum LLMClientType {
    Anthropic(Box<AnthropicClient>),
    Gemini(Box<GeminiClient>),
}

impl LLMClientType {
    pub fn new(
        client_type: ClientType,
        model: &str,
        streaming: bool,
        tools: Option<Value>,
    ) -> Result<Self> {
        let config = LLMConfig {
            api_key: match client_type {
                ClientType::Anthropic => {
                    std::env::var("ANTHROPIC_API_KEY_RS").context("Missing ANTHROPIC_API_KEY_RS")?
                }
                ClientType::Gemini => {
                    std::env::var("GEMINI_API_KEY").context("Missing GEMINI_API_KEY")?
                }
            },
            model: model.to_string(),
            temperature: None,
            max_tokens: Some(4000),
            streaming,
            system_prompt: None,
            tools,
            stop_sequences: None,
            top_p: None,
            top_k: None,
        };

        match client_type {
            ClientType::Anthropic => {
                let mut client = AnthropicClient::with_config(config)?;
                // Add Anthropic-specific configuration
                client.clone().with_beta("prompt-caching-2024-07-31");
                Ok(LLMClientType::Anthropic(Box::new(client)))
            }
            ClientType::Gemini => {
                let client = GeminiClient::with_config(config)?;
                Ok(LLMClientType::Gemini(Box::new(client)))
            }
        }
    }

    pub async fn send_message(&self, content: &str) -> Result<String> {
        match self {
            LLMClientType::Anthropic(client) => client.send_message(content).await,
            LLMClientType::Gemini(client) => client.send_message(content).await,
        }
    }

    pub async fn stream_message<F, Fut>(&self, content: &str, callback: F) -> Result<()>
    where
        F: FnMut(String) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        match self {
            LLMClientType::Anthropic(client) => client.stream_message(content, callback).await,
            LLMClientType::Gemini(client) => client.stream_message(content, callback).await,
        }
    }

    pub async fn send_message_raw(&self, content: &str) -> Result<Value> {
        match self {
            LLMClientType::Anthropic(client) => client.send_message_raw(content).await,
            LLMClientType::Gemini(client) => client.send_message_raw(content).await,
        }
    }

    pub fn update_config(&mut self, config: LLMConfig) -> Result<()> {
        match self {
            LLMClientType::Anthropic(client) => client.update_config(config),
            LLMClientType::Gemini(client) => client.update_config(config),
        }
    }
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_switching() -> Result<()> {
        let client = LLMClientType::new(ClientType::Anthropic, "claude-3", false, None)?;

        let response = client.send_message("Hello!").await?;
        println!("Response: {}", response);

        Ok(())
    }
}
