use crate::types::{
    GeminiCandidate, GeminiContent, GeminiError, GeminiFunctionCall, GeminiFunctionDeclaration,
    GeminiFunctionResponse, GeminiGenerationConfig, GeminiPart, GeminiRequest, GeminiResponse,
    GeminiSafetySetting, GeminiTool,
};
use anyhow::{anyhow, Context, Result};
use reqwest::{Client as ReqwestClient, Error as ReqwestError, StatusCode};
use serde::Deserialize;
use serde_json::Value;

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";

#[derive(Debug, Clone)]
pub struct GeminiClient {
    client: ReqwestClient,
    api_key: String,
    model: String,
    temperature: Option<f32>,
    top_p: Option<f32>,
    top_k: Option<i32>,
    max_output_tokens: Option<i32>,
    stop_sequences: Option<Vec<String>>,
    tools: Option<Vec<GeminiTool>>,
    safety_settings: Option<Vec<GeminiSafetySetting>>,
    stream: bool,
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

impl GeminiClient {
    pub fn new() -> Self {
        Self {
            client: ReqwestClient::new(),
            api_key: String::new(),
            model: "gemini-2.0-flash-exp".to_string(),
            temperature: None,
            top_p: None,
            top_k: None,
            max_output_tokens: None,
            stop_sequences: None,
            tools: None,
            safety_settings: None,
            stream: false,
        }
    }

    pub fn auth(mut self, api_key: &str) -> Self {
        self.api_key = api_key.to_owned();
        self
    }

    pub fn model(mut self, model: &str) -> Self {
        self.model = model.to_owned();
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn top_k(mut self, top_k: i32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    pub fn max_output_tokens(mut self, max_output_tokens: i32) -> Self {
        self.max_output_tokens = Some(max_output_tokens);
        self
    }

    pub fn stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(stop_sequences);
        self
    }

    pub fn tools(mut self, tools: Vec<GeminiTool>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn safety_settings(mut self, safety_settings: Vec<GeminiSafetySetting>) -> Self {
        self.safety_settings = Some(safety_settings);
        self
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    pub fn build(self) -> Result<GeminiRequest, ReqwestError> {
        let generation_config = GeminiGenerationConfig {
            temperature: self.temperature,
            top_p: self.top_p,
            top_k: self.top_k,
            max_output_tokens: self.max_output_tokens,
            stop_sequences: self.stop_sequences,
        };

        let request = GeminiRequest {
            contents: Vec::new(), // To be filled by the caller
            tools: self.tools,
            safety_settings: self.safety_settings,
            generation_config: Some(generation_config),
        };

        Ok(request)
    }

    pub async fn generate(&mut self, contents: Vec<GeminiContent>) -> Result<GeminiResponse> {
        let mut request = self.clone().build()?;
        request.contents = contents;

        let url = format!(
            "{}/{}:generateContent?key={}",
            GEMINI_API_BASE, self.model, self.api_key
        );

        dbg!(&request);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request")?;

        match response.status() {
            StatusCode::OK => {
                let response_text = response
                    .text()
                    .await
                    .context("Failed to read response text")?;

                let gemini_response: GeminiResponse = serde_json::from_str(&response_text)
                    .context("Failed to parse response as GeminiResponse")?;
                Ok(gemini_response)
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

    pub async fn generate_stream<F, Fut>(
        &self,
        contents: Vec<GeminiContent>,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(String) -> Fut,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let mut request = self.clone().build()?;
        request.contents = contents;

        let url = format!(
            "{}/{}/streamGenerateContent?key={}",
            GEMINI_API_BASE, self.model, self.api_key
        );

        let mut response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request")?;

        dbg!(&response);

        match response.status() {
            StatusCode::OK => {
                while let Some(chunk) = response.chunk().await? {
                    let chunk_str = String::from_utf8(chunk.to_vec())
                        .context("Failed to decode chunk as UTF-8")?;

                    if chunk_str.trim().is_empty() {
                        continue;
                    }

                    let chunk_response: GeminiResponse = serde_json::from_str(&chunk_str)
                        .context("Failed to parse chunk as GeminiResponse")?;

                    for candidate in chunk_response.candidates {
                        if let Some(text) = Self::extract_text_from_candidate(&candidate) {
                            callback(text).await;
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

    fn extract_text_from_candidate(candidate: &GeminiCandidate) -> Option<String> {
        for part in &candidate.content.parts {
            if let GeminiPart::Text { text } = part {
                return Some(text.clone());
            }
        }
        None
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
