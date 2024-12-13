use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub api_key: String,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub streaming: bool,
    pub system_prompt: Option<String>,
    pub tools: Option<Value>,
    pub stop_sequences: Option<Vec<String>>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
}

#[async_trait]
pub trait LLMClient: Send + Sync {
    /// Send a message and get a text response
    async fn send_message(&self, content: &str) -> Result<String>;

    /// Stream a response with callback
    async fn stream_message<F, Fut>(&self, content: &str, callback: F) -> Result<()>
    where
        F: FnMut(String) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static;

    /// Send a message and get raw JSON response
    async fn send_message_raw(&self, content: &str) -> Result<Value>;

    /// Configure the client
    fn with_config(config: LLMConfig) -> Result<Self>
    where
        Self: Sized;

    /// Update configuration
    fn update_config(&mut self, config: LLMConfig) -> Result<()>;
}
