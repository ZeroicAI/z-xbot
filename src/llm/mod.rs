use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::warn;

pub struct LlmClient {
    client: Client,
    api_key: String,
}

#[derive(Serialize)]
struct ApiMessage {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct ApiRequest {
    model: &'static str,
    max_tokens: u32,
    messages: Vec<ApiMessage>,
}

#[derive(Deserialize)]
struct ApiResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

impl LlmClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn complete(&self, prompt: &str) -> Result<String> {
        let request = ApiRequest {
            model: "claude-haiku-4-5-20251001",
            max_tokens: 300,
            messages: vec![ApiMessage {
                role: "user",
                content: prompt.to_string(),
            }],
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Claude API error {}: {}", status, body);
            anyhow::bail!("Claude API error {}: {}", status, body);
        }

        let parsed = response.json::<ApiResponse>().await?;
        Ok(parsed.content
            .into_iter()
            .next()
            .map(|b| b.text)
            .unwrap_or_default()
            .trim()
            .to_string())
    }
}
