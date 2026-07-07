use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use tracing::{info, warn};

const KEYWORDS: &[&str] = &[
    "ai", "agent", "llm", "gpt", "claude", "openai", "anthropic", "gemini",
    "rust", "cargo", "tokio", "wasm",
    "solana", "crypto", "defi", "blockchain", "web3", "token",
    "autonomous", "multi-agent", "swarm", "neural", "transformer",
    "machine learning", "deep learning", "robotics",
];

#[derive(Deserialize)]
struct HnItem {
    title: Option<String>,
    score: Option<i64>,
}

pub async fn fetch_trending_topic(client: &Client) -> Option<String> {
    match fetch_hn(client).await {
        Ok(Some(topic)) => {
            info!("Scout: HN trending topic → \"{}\"", topic);
            Some(topic)
        }
        Ok(None) => {
            info!("Scout: no relevant HN topics found, using predefined queue");
            None
        }
        Err(e) => {
            warn!("Scout: HN fetch failed ({}), using predefined queue", e);
            None
        }
    }
}

async fn fetch_hn(client: &Client) -> Result<Option<String>> {
    let ids: Vec<u64> = client
        .get("https://hacker-news.firebaseio.com/v0/topstories.json")
        .send()
        .await?
        .json()
        .await?;

    for id in ids.iter().take(50) {
        let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);

        let item: HnItem = match client.get(&url).send().await {
            Ok(r) => match r.json().await {
                Ok(i) => i,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        let title = match item.title {
            Some(t) if !t.is_empty() => t,
            _ => continue,
        };

        let lower = title.to_lowercase();
        let relevant = KEYWORDS.iter().any(|kw| lower.contains(kw));
        let score = item.score.unwrap_or(0);

        if relevant && score >= 50 {
            return Ok(Some(title));
        }
    }

    Ok(None)
}
