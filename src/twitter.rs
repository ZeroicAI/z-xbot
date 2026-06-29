use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;

pub struct TwitterClient {
    client: Client,
    consumer_key: String,
    consumer_secret: String,
    access_token: String,
    access_token_secret: String,
}

#[derive(Serialize)]
struct TweetRequest {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply: Option<ReplyTo>,
}

#[derive(Serialize)]
struct ReplyTo {
    in_reply_to_tweet_id: String,
}

#[derive(Deserialize, Debug)]
pub struct TweetResponse {
    pub data: TweetData,
}

#[derive(Deserialize, Debug)]
pub struct TweetData {
    pub id: String,
    #[allow(dead_code)]
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct MentionsResponse {
    #[serde(default)]
    pub data: Vec<MentionData>,
    pub meta: Option<MentionsMeta>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MentionData {
    pub id: String,
    pub text: String,
    pub author_id: String,
}

#[derive(Deserialize, Debug)]
pub struct MentionsMeta {
    pub newest_id: Option<String>,
    #[allow(dead_code)]
    pub result_count: u64,
}

#[derive(Deserialize, Debug)]
struct UserLookupResponse {
    data: UserData,
}

#[derive(Deserialize, Debug)]
struct UserData {
    id: String,
}

/// RFC 3986 percent-encoding for OAuth 1.0a
fn percent_encode(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

impl TwitterClient {
    pub fn new() -> Result<Self> {
        let consumer_key = env::var("TWITTER_CONSUMER_KEY")
            .map_err(|_| anyhow::anyhow!("TWITTER_CONSUMER_KEY not set"))?;
        let consumer_secret = env::var("TWITTER_CONSUMER_SECRET")
            .map_err(|_| anyhow::anyhow!("TWITTER_CONSUMER_SECRET not set"))?;
        let access_token = env::var("TWITTER_ACCESS_TOKEN")
            .map_err(|_| anyhow::anyhow!("TWITTER_ACCESS_TOKEN not set"))?;
        let access_token_secret = env::var("TWITTER_ACCESS_TOKEN_SECRET")
            .map_err(|_| anyhow::anyhow!("TWITTER_ACCESS_TOKEN_SECRET not set"))?;

        Ok(Self {
            client: Client::new(),
            consumer_key,
            consumer_secret,
            access_token,
            access_token_secret,
        })
    }

    /// Post a new tweet
    pub async fn post_tweet(&self, text: &str) -> Result<TweetResponse> {
        let url = "https://api.x.com/2/tweets";

        let tweet_request = TweetRequest {
            text: text.to_string(),
            reply: None,
        };

        let auth_header = self.create_oauth_header("POST", url, None)?;

        let response = self
            .client
            .post(url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .json(&tweet_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("Twitter API error ({}): {}", status, error_text);
        }

        let tweet_response = response.json::<TweetResponse>().await?;
        Ok(tweet_response)
    }

    /// Reply to a specific tweet
    pub async fn reply_to_tweet(&self, tweet_id: &str, text: &str) -> Result<TweetResponse> {
        let url = "https://api.x.com/2/tweets";

        let tweet_request = TweetRequest {
            text: text.to_string(),
            reply: Some(ReplyTo {
                in_reply_to_tweet_id: tweet_id.to_string(),
            }),
        };

        let auth_header = self.create_oauth_header("POST", url, None)?;

        let response = self
            .client
            .post(url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .json(&tweet_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("Twitter reply error ({}): {}", status, error_text);
        }

        let tweet_response = response.json::<TweetResponse>().await?;
        Ok(tweet_response)
    }

    /// Get recent mentions for a user
    pub async fn get_mentions(
        &self,
        user_id: &str,
        since_id: Option<&str>,
    ) -> Result<MentionsResponse> {
        let base_url = format!("https://api.x.com/2/users/{}/mentions", user_id);

        let mut query_params = BTreeMap::new();
        query_params.insert("max_results".to_string(), "10".to_string());
        query_params.insert(
            "tweet.fields".to_string(),
            "author_id,text".to_string(),
        );

        if let Some(sid) = since_id {
            query_params.insert("since_id".to_string(), sid.to_string());
        }

        let query_string: String = query_params
            .iter()
            .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let full_url = format!("{}?{}", base_url, query_string);

        let auth_header =
            self.create_oauth_header("GET", &base_url, Some(&query_params))?;

        let response = self
            .client
            .get(&full_url)
            .header("Authorization", auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("Twitter mentions error ({}): {}", status, error_text);
        }

        let mentions = response.json::<MentionsResponse>().await?;
        Ok(mentions)
    }

    /// Look up user ID from username
    pub async fn get_user_id(&self, username: &str) -> Result<String> {
        let base_url = format!("https://api.x.com/2/users/by/username/{}", username);

        let auth_header = self.create_oauth_header("GET", &base_url, None)?;

        let response = self
            .client
            .get(&base_url)
            .header("Authorization", auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("User lookup error ({}): {}", status, error_text);
        }

        let user: UserLookupResponse = response.json().await?;
        Ok(user.data.id)
    }

    fn create_oauth_header(
        &self,
        method: &str,
        url: &str,
        extra_params: Option<&BTreeMap<String, String>>,
    ) -> Result<String> {
        use base64::{engine::general_purpose, Engine as _};
        use chrono::Utc;
        use hmac::{Hmac, Mac};
        use sha1::Sha1;

        type HmacSha1 = Hmac<Sha1>;

        let timestamp = Utc::now().timestamp().to_string();
        let nonce: String = rand::random::<u64>().to_string();

        // Collect all params for signing (must be sorted)
        let mut all_params = BTreeMap::new();
        all_params.insert("oauth_consumer_key".to_string(), self.consumer_key.clone());
        all_params.insert("oauth_nonce".to_string(), nonce.clone());
        all_params.insert(
            "oauth_signature_method".to_string(),
            "HMAC-SHA1".to_string(),
        );
        all_params.insert("oauth_timestamp".to_string(), timestamp.clone());
        all_params.insert("oauth_token".to_string(), self.access_token.clone());
        all_params.insert("oauth_version".to_string(), "1.0".to_string());

        // Include query params in signature
        if let Some(extra) = extra_params {
            for (k, v) in extra {
                all_params.insert(k.clone(), v.clone());
            }
        }

        // Create parameter string
        let param_string = all_params
            .iter()
            .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        // Signature base string
        let signature_base = format!(
            "{}&{}&{}",
            method,
            percent_encode(url),
            percent_encode(&param_string)
        );

        // Signing key
        let signing_key = format!(
            "{}&{}",
            percent_encode(&self.consumer_secret),
            percent_encode(&self.access_token_secret)
        );

        // HMAC-SHA1 signature
        let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes())
            .map_err(|e| anyhow::anyhow!("HMAC error: {}", e))?;
        mac.update(signature_base.as_bytes());
        let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        // Build authorization header
        let auth_header = format!(
            r#"OAuth oauth_consumer_key="{}", oauth_nonce="{}", oauth_signature="{}", oauth_signature_method="HMAC-SHA1", oauth_timestamp="{}", oauth_token="{}", oauth_version="1.0""#,
            percent_encode(&self.consumer_key),
            percent_encode(&nonce),
            percent_encode(&signature),
            timestamp,
            percent_encode(&self.access_token)
        );

        Ok(auth_header)
    }
}
