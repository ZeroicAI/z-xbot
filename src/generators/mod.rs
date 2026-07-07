use crate::templates::TweetTemplates;
use crate::config::{BotConfig, ContentCategory};
use rand::seq::SliceRandom;
use tracing::warn;

/// Maximum tweet length (Twitter limit)
const MAX_TWEET_LENGTH: usize = 280;

pub struct TweetGenerator;

impl TweetGenerator {
    /// Generate a random bull post based on enabled categories
    pub fn generate(config: &BotConfig) -> String {
        let enabled_categories = config.get_enabled_categories();

        if enabled_categories.is_empty() {
            return TweetTemplates::random_bull_tweet();
        }

        // Pick random category from enabled ones
        let category = enabled_categories
            .choose(&mut rand::thread_rng())
            .unwrap();

        match category {
            ContentCategory::AI => TweetTemplates::random_ai_tweet(),
            ContentCategory::ZeroicAI => TweetTemplates::random_zeroicai_tweet(),
            ContentCategory::Crypto => TweetTemplates::random_crypto_tweet(),
            ContentCategory::Meme => TweetTemplates::random_meme_tweet(),
        }
    }

    /// Pick a random agent name for the signature
    fn agent_name() -> &'static str {
        let agents = ["ZERO", "AXIOM", "NEXUS", "CIPHER", "VECTOR"];
        agents.choose(&mut rand::thread_rng()).unwrap()
    }

    /// Add agent signature, but only if the result fits within 280 chars
    pub fn add_signature(tweet: String, _username: &str) -> String {
        let agent = Self::agent_name();
        let signature = format!("\n\n↳ Agent {} · zeroicai.xyz", agent);
        let with_sig = format!("{}{}", tweet, signature);

        if with_sig.len() <= MAX_TWEET_LENGTH {
            with_sig
        } else {
            warn!(
                "Tweet is {} chars; skipping signature to stay within {} limit",
                with_sig.len(),
                MAX_TWEET_LENGTH
            );
            tweet
        }
    }

    /// Generate and prepare tweet for posting
    pub fn create_tweet(config: &BotConfig) -> String {
        let base_tweet = Self::generate(config);
        Self::add_signature(base_tweet, &config.bot_username)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> BotConfig {
        BotConfig {
            bot_username: "test".to_string(),
            post_interval_hours: 6,
            max_posts_per_day: 4,
            enable_crypto: true,
            enable_meme: true,
            enable_ai: true,
            enable_zeroicai: true,
            enable_replies: false,
            mention_poll_seconds: 300,
            twitter_user_id: None,
        }
    }

    #[test]
    fn test_generate_produces_content() {
        let config = test_config();
        let tweet = TweetGenerator::generate(&config);
        assert!(!tweet.is_empty());
    }

    #[test]
    fn test_signature_respects_length() {
        let long_tweet = "a".repeat(270);
        let result = TweetGenerator::add_signature(long_tweet.clone(), "testbot");
        assert_eq!(result, long_tweet);
    }

    #[test]
    fn test_create_tweet_within_limit() {
        let config = test_config();
        let tweet = TweetGenerator::create_tweet(&config);
        assert!(tweet.len() <= 280);
    }
}
