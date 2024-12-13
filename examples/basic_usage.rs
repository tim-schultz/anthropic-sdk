// examples/basic_usage.rs

use anthropic_sdk::{AnthropicClient, ClientType, LLMClientType};
use dotenv::dotenv;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let secret_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();

    let client = LLMClientType::new(ClientType::Anthropic, "claude-3-opus-20240229", false, None)?;

    let request = client.send_message("Write me a poem about bravery").await?;

    let request = client
        .messages(&json!([
            {"role": "user", "content": "Write me a poem about bravery"}
        ]))
        .max_tokens(1024)
        .build()?;

    if let Err(error) = request
        .execute(|text| async move { println!("{text}") })
        .await
    {
        eprintln!("Error: {error}");
    }

    Ok(())
}
