use anyhow::{anyhow, Context, Result};
use reqwest::{Client as ReqwestClient, Error as ReqwestError, RequestBuilder, StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};
use types::AnthropicChatCompletionChunk;
mod types;
use std::collections::HashMap;

use crate::types::AnthropicErrorMessage;

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
pub struct Client {
    client: ReqwestClient,
    secret_key: String,
    model: String,
    messages: Value,
    tools: Value,
    metadata: Value,
    max_tokens: i32,
    stream: bool,
    verbose: bool,
    temperature: f32,
    system: String,
    version: String,
    stop_sequences: Vec<String>,
    beta: Option<String>,
    top_k: Option<i32>,
    top_p: Option<f64>,
    tool_choice: Option<Value>,
}

#[derive(Deserialize)]
struct JsonResponse {
    content: Vec<Content>,
}

#[derive(Deserialize)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

impl Client {
    pub fn new() -> Self {
        Self {
            client: ReqwestClient::new(),
            secret_key: String::new(),
            model: String::new(),
            messages: Value::Null,
            tools: Value::Null,
            metadata: Value::Null,
            max_tokens: 1024,
            stream: false,
            verbose: false,
            temperature: 0.0,
            system: String::new(),
            version: "2023-06-01".to_string(),
            stop_sequences: Vec::new(),
            beta: None,
            top_k: None,
            top_p: None,
            tool_choice: None,
        }
    }

    pub fn auth(mut self, secret_key: &str) -> Self {
        self.secret_key = secret_key.to_owned();
        self
    }

    pub fn model(mut self, model: &str) -> Self {
        self.model = model.to_owned();
        self
    }

    pub fn messages(mut self, messages: &Value) -> Self {
        self.messages = messages.clone();
        self
    }

    pub fn tools(mut self, tools: &Value) -> Self {
        self.tools = tools.clone();
        self
    }

    pub fn metadata(mut self, metadata: &Value) -> Self {
        self.metadata = metadata.clone();
        self
    }

    pub fn max_tokens(mut self, max_tokens: i32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature.to_owned();
        self
    }

    pub fn system(mut self, system: &str) -> Self {
        self.system = system.to_owned();
        self
    }
    pub fn version(mut self, version: &str) -> Self {
        self.version = version.to_owned();
        self
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn beta(mut self, beta: &str) -> Self {
        self.beta = Some(beta.to_owned());
        self
    }

    pub fn stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.stop_sequences = stop_sequences;
        self
    }

    pub fn top_k(mut self, top_k: i32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    pub fn top_p(mut self, top_p: f64) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn tool_choice(mut self, tool_choice: Value) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    pub fn build(self) -> Result<Request, ReqwestError> {
        let mut body_map: HashMap<&str, Value> = HashMap::new();
        body_map.insert("model", json!(self.model));
        body_map.insert("max_tokens", json!(self.max_tokens));
        body_map.insert("messages", json!(self.messages));
        body_map.insert("stream", json!(self.stream));
        body_map.insert("temperature", json!(self.temperature));
        body_map.insert(
            "system",
            json!([{
                "type": "text",
                "text": self.system,
                "cache_control": {"type": "ephemeral"}
            }]),
        );

        if let Some(tool_choice) = self.tool_choice {
            body_map.insert("tool_choice", tool_choice);
        }

        if self.tools != Value::Null {
            body_map.insert("tools", self.tools.clone());
        }

        if self.metadata != Value::Null {
            body_map.insert("metadata", self.metadata.clone());
        }

        if self.stop_sequences.len() > 0 {
            body_map.insert("stop_sequences", json!(self.stop_sequences));
        }

        if let Some(top_k) = self.top_k {
            body_map.insert("top_k", json!(top_k));
        }

        if let Some(top_p) = self.top_p {
            body_map.insert("top_p", json!(top_p));
        }

        let mut request_builder = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", self.secret_key)
            .header("anthropic-version", self.version)
            .header("content-type", "application/json")
            .json(&body_map);

        if let Some(beta_value) = self.beta {
            request_builder = request_builder.header("anthropic-beta", beta_value);
        }

        Ok(Request {
            request_builder,
            stream: self.stream,
            verbose: self.verbose,
            tools: self.tools,
        })
    }

    pub fn builder(self) -> Result<RequestBuilder, ReqwestError> {
        let mut body_map: HashMap<&str, Value> = HashMap::new();
        body_map.insert("model", json!(self.model));
        body_map.insert("max_tokens", json!(self.max_tokens));
        body_map.insert("messages", json!(self.messages));
        body_map.insert("stream", json!(self.stream));
        body_map.insert("temperature", json!(self.temperature));
        body_map.insert("system", json!(self.system));

        if self.tools != Value::Null {
            body_map.insert("tools", self.tools.clone());
        }

        if self.metadata != Value::Null {
            body_map.insert("metadata", self.metadata.clone());
        }

        if self.stop_sequences.len() > 0 {
            body_map.insert("stop_sequences", json!(self.stop_sequences));
        }

        if let Some(top_k) = self.top_k {
            body_map.insert("top_k", json!(top_k));
        }

        if let Some(top_p) = self.top_p {
            body_map.insert("top_p", json!(top_p));
        }

        let mut request_builder = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", self.secret_key)
            .header("anthropic-version", self.version)
            .header("content-type", "application/json")
            .json(&body_map);

        if let Some(beta_value) = self.beta {
            request_builder = request_builder.header("anthropic-beta", beta_value);
        }

        Ok(request_builder)
    }
}

pub struct Request {
    request_builder: RequestBuilder,
    stream: bool,
    verbose: bool,
    tools: Value,
}

impl Request {
    pub async fn execute<F, Fut>(self, mut callback: F) -> Result<()>
    where
        F: FnMut(String) -> Fut,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let mut response = self
            .request_builder
            .send()
            .await
            .context("Failed to send request")?;

        match response.status() {
            StatusCode::OK => {
                if self.stream {
                    let mut buffer = String::new();
                    while let Some(chunk) = response.chunk().await? {
                        let s = match std::str::from_utf8(&chunk) {
                            Ok(v) => v,
                            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                        };
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
                                    let processed_chunk = chunk
                                        .trim_start_matches("event: message_start")
                                        .trim_start_matches("event: content_block_start")
                                        .trim_start_matches("event: ping")
                                        .trim_start_matches("event: content_block_delta")
                                        .trim_start_matches("event: content_block_stop")
                                        .trim_start_matches("event: message_delta")
                                        .trim_start_matches("event: message_stop")
                                        .to_string();
                                    let cleaned_string = &processed_chunk
                                        .trim_start()
                                        .strip_prefix("data: ")
                                        .unwrap_or(&processed_chunk);
                                    match serde_json::from_str::<AnthropicChatCompletionChunk>(
                                        &cleaned_string,
                                    ) {
                                        Ok(d) => {
                                            if let Some(delta) = d.delta {
                                                if let Some(content) = delta.text {
                                                    callback(content).await;
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            let processed_chunk = cleaned_string
                                                .trim_start_matches("event: error")
                                                .to_string();
                                            let cleaned_string = &processed_chunk
                                                .trim_start()
                                                .strip_prefix("data: ")
                                                .unwrap_or(&processed_chunk);
                                            match serde_json::from_str::<AnthropicErrorMessage>(
                                                &cleaned_string,
                                            ) {
                                                Ok(error_message) => {
                                                    return Err(anyhow!(
                                                        "{}: {}",
                                                        error_message.error.error_type,
                                                        error_message.error.message
                                                    ));
                                                }
                                                Err(_) => {
                                                    eprintln!(
                                                        "Couldn't parse AnthropicChatCompletionChunk or AnthropicErrorMessage: {}",
                                                        &cleaned_string
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                    }
                } else {
                    let json_text = response
                        .text()
                        .await
                        .context("Failed to read response text")?;
                    if self.tools == Value::Null && !self.verbose {
                        match serde_json::from_str::<JsonResponse>(&json_text) {
                            Ok(parsed_json) => {
                                if let Some(content) = parsed_json
                                    .content
                                    .iter()
                                    .find(|c| c.content_type == "text")
                                {
                                    callback(content.text.clone()).await;
                                }
                            }
                            Err(_) => return Err(anyhow!("Unable to parse JSON")),
                        }
                    } else {
                        callback(json_text).await;
                    }
                }
                Ok(())
            }
            StatusCode::BAD_REQUEST => Err(anyhow!(
                "Bad request. Check your request parameters. {}",
                response.text().await?
            )),
            StatusCode::UNAUTHORIZED => Err(anyhow!("Unauthorized. Check your authorization key.")),
            StatusCode::TOO_MANY_REQUESTS => {
                Err(anyhow!("Too many Requests. You have been rate limited."))
            }
            _ => {
                let error_message = format!("Unexpected status code: {:?}", response.text().await?);
                Err(anyhow!(error_message))
            }
        }
    }
    pub async fn execute_and_return_json(self) -> Result<AnthropicResponse> {
        let response = self
            .request_builder
            .send()
            .await
            .context("Failed to send request")?;

        match response.status() {
            StatusCode::OK => {
                let json_text = response
                    .text()
                    .await
                    .context("Failed to read response text")?;

                let anthropic_response: AnthropicResponse = serde_json::from_str(&json_text)
                    .context("Failed to parse response as AnthropicResponse")?;
                Ok(anthropic_response)
            }
            StatusCode::BAD_REQUEST => {
                let error_text = response.text().await?;
                Err(anyhow!(
                    "Bad request. Check your request parameters. {}",
                    error_text
                ))
            }
            StatusCode::UNAUTHORIZED => Err(anyhow!("Unauthorized. Check your authorization key.")),
            StatusCode::TOO_MANY_REQUESTS => {
                Err(anyhow!("Too many Requests. You have been rate limited."))
            }
            _ => {
                let error_message = format!("Unexpected status code: {}", response.status());
                let error_text = response.text().await?;
                Err(anyhow!("{}: {}", error_message, error_text))
            }
        }
    }
}
