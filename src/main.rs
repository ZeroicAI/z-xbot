mod generators;
mod filters;
mod config;
mod twitter;
mod knowledge;
mod responder;

use anyhow::Result;
use dotenv::dotenv;
use tokio_cron_scheduler::{JobScheduler, Job};
use tracing::{info, warn, error};
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{Utc, NaiveDate};

use z_cognition::{BeliefBase, ReasoningEngine};

use generators::{TweetGenerator, TopicQueue};
use filters::ContentFilter;
use config::BotConfig;
use twitter::TwitterClient;
use knowledge::build_knowledge_base;
use responder::{build_reasoning_engine, generate_response};

/// Tracks daily post count and resets each day
struct PostTracker {
    count: u32,
    day: NaiveDate,
    max_per_day: u32,
}

impl PostTracker {
    fn new(max_per_day: u32) -> Self {
        Self {
            count: 0,
            day: Utc::now().date_naive(),
            max_per_day,
        }
    }

    fn try_post(&mut self) -> bool {
        let today = Utc::now().date_naive();
        if today != self.day {
            self.count = 0;
            self.day = today;
        }
        if self.count >= self.max_per_day {
            return false;
        }
        self.count += 1;
        true
    }
}

/// Tracks the last processed mention ID to avoid duplicates
struct MentionTracker {
    last_seen_id: Option<String>,
}

impl MentionTracker {
    fn new() -> Self {
        Self { last_seen_id: None }
    }
}

/// Shared brain: knowledge + reasoning engine
struct AgentBrain {
    beliefs: BeliefBase,
    engine: ReasoningEngine,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("ZeroicAI Bot starting...");

    let config = BotConfig::from_env()?;
    config.validate()?;

    info!("Bot Configuration:");
    info!("  Username: {}", config.bot_username);
    info!("  Post Interval: {} hours", config.post_interval_hours);
    info!("  Max Posts/Day: {}", config.max_posts_per_day);
    info!("  Replies Enabled: {}", config.enable_replies);
    if config.enable_replies {
        info!("  Mention Poll: every {} seconds", config.mention_poll_seconds);
    }

    let twitter_client = Arc::new(TwitterClient::new()?);
    info!("Twitter client initialized");

    let brain = Arc::new(AgentBrain {
        beliefs: build_knowledge_base(),
        engine: build_reasoning_engine(),
    });
    info!("Agent brain loaded: {} beliefs", "knowledge base + reasoning engine");

    let tracker = Arc::new(Mutex::new(PostTracker::new(config.max_posts_per_day)));
    let mention_tracker = Arc::new(Mutex::new(MentionTracker::new()));
    let topic_queue = Arc::new(Mutex::new(TopicQueue::new()));

    let scheduler = JobScheduler::new().await?;

    // --- Tweet posting job ---
    let cron_expr = config.get_cron_expression();
    info!("Tweet cron schedule: {}", cron_expr);

    let config_clone = config.clone();
    let client_clone = Arc::clone(&twitter_client);
    let tracker_clone = Arc::clone(&tracker);
    let queue_clone = Arc::clone(&topic_queue);
    let brain_clone = Arc::clone(&brain);

    let tweet_job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
        let config_inner = config_clone.clone();
        let client_inner = Arc::clone(&client_clone);
        let tracker_inner = Arc::clone(&tracker_clone);
        let queue_inner = Arc::clone(&queue_clone);
        let brain_inner = Arc::clone(&brain_clone);
        Box::pin(async move {
            if let Err(e) = post_tweet(&client_inner, &config_inner, &tracker_inner, &queue_inner, &brain_inner).await {
                error!("Failed to post tweet: {}", e);
            }
        })
    })?;

    scheduler.add(tweet_job).await?;

    // --- Mention reply job ---
    if config.enable_replies {
        let user_id = match &config.twitter_user_id {
            Some(id) => {
                info!("Using configured user ID: {}", id);
                id.clone()
            }
            None => {
                info!("Looking up user ID for @{}...", config.bot_username);
                match twitter_client.get_user_id(&config.bot_username).await {
                    Ok(id) => {
                        info!("Resolved user ID: {}", id);
                        id
                    }
                    Err(e) => {
                        error!("Failed to resolve user ID: {}. Replies disabled.", e);
                        String::new()
                    }
                }
            }
        };

        if !user_id.is_empty() {
            let mention_cron = config.get_mention_cron();
            info!("Mention poll cron: {}", mention_cron);

            let client_mention = Arc::clone(&twitter_client);
            let brain_mention = Arc::clone(&brain);
            let mention_tracker_clone = Arc::clone(&mention_tracker);
            let user_id_clone = user_id.clone();

            let mention_job = Job::new_async(mention_cron.as_str(), move |_uuid, _lock| {
                let client_inner = Arc::clone(&client_mention);
                let brain_inner = Arc::clone(&brain_mention);
                let tracker_inner = Arc::clone(&mention_tracker_clone);
                let uid = user_id_clone.clone();
                Box::pin(async move {
                    if let Err(e) = check_and_reply_mentions(
                        &client_inner,
                        &brain_inner,
                        &tracker_inner,
                        &uid,
                    )
                    .await
                    {
                        error!("Failed to process mentions: {}", e);
                    }
                })
            })?;

            scheduler.add(mention_job).await?;
            info!("Mention polling active for user ID: {}", user_id);
        }
    }

    // Post one immediately on startup
    info!("Posting initial tweet...");
    post_tweet(&twitter_client, &config, &tracker, &topic_queue, &brain).await?;

    scheduler.start().await?;

    info!("Bot is now running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}

async fn post_tweet(
    client: &TwitterClient,
    config: &BotConfig,
    tracker: &Arc<Mutex<PostTracker>>,
    topic_queue: &Arc<Mutex<TopicQueue>>,
    brain: &AgentBrain,
) -> Result<()> {
    {
        let mut t = tracker.lock().await;
        if !t.try_post() {
            warn!("Daily post limit ({}) reached, skipping", config.max_posts_per_day);
            return Ok(());
        }
        info!("Post {}/{} for today", t.count, t.max_per_day);
    }

    let topic = {
        let mut queue = topic_queue.lock().await;
        queue.next()
    };

    info!("Generating tweet for topic: {:?}", topic);

    let tweet = match TweetGenerator::create_tweet(&topic, &brain.beliefs) {
        Some(t) => t,
        None => {
            error!("Failed to compose tweet for topic {:?}", topic);
            return Ok(());
        }
    };

    let preview = tweet.chars().take(60).collect::<String>();
    info!("Tweet preview: {}...", preview);

    const MAX_RETRIES: u32 = 3;
    let mut last_error = None;

    for attempt in 1..=MAX_RETRIES {
        match client.post_tweet(&tweet).await {
            Ok(response) => {
                info!("Tweet posted successfully! ID: {}", response.data.id);
                return Ok(());
            }
            Err(e) => {
                warn!("Tweet attempt {}/{} failed: {}", attempt, MAX_RETRIES, e);
                last_error = Some(e);
                if attempt < MAX_RETRIES {
                    let backoff = std::time::Duration::from_secs(2u64.pow(attempt));
                    info!("Retrying in {:?}...", backoff);
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }

    Err(last_error.unwrap())
}

async fn check_and_reply_mentions(
    client: &TwitterClient,
    brain: &AgentBrain,
    mention_tracker: &Arc<Mutex<MentionTracker>>,
    user_id: &str,
) -> Result<()> {
    let since_id = {
        let tracker = mention_tracker.lock().await;
        tracker.last_seen_id.clone()
    };

    info!("Checking mentions (since: {:?})...", since_id);

    let mentions = client
        .get_mentions(user_id, since_id.as_deref())
        .await?;

    let count = mentions.data.len();
    if count == 0 {
        info!("No new mentions");
        return Ok(());
    }

    info!("Found {} new mention(s)", count);

    if let Some(meta) = &mentions.meta {
        if let Some(newest) = &meta.newest_id {
            let mut tracker = mention_tracker.lock().await;
            tracker.last_seen_id = Some(newest.clone());
            info!("Updated last_seen_id to {}", newest);
        }
    }

    for mention in mentions.data.iter().rev() {
        info!(
            "Processing mention {} from user {}: \"{}\"",
            mention.id, mention.author_id, mention.text
        );

        let response = generate_response(&mention.text, &brain.beliefs, &brain.engine);

        match response {
            Some(reply_text) => {
                let validated = match ContentFilter::validate(reply_text) {
                    Some(t) => t,
                    None => {
                        warn!("Reply failed content filter, skipping mention {}", mention.id);
                        continue;
                    }
                };

                info!("Replying to {}: \"{}\"", mention.id, &validated[..validated.len().min(50)]);

                match client.reply_to_tweet(&mention.id, &validated).await {
                    Ok(response) => {
                        info!("Reply posted! ID: {}", response.data.id);
                    }
                    Err(e) => {
                        error!("Failed to reply to {}: {}", mention.id, e);
                    }
                }

                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
            None => {
                warn!("Could not generate response for mention {}", mention.id);
            }
        }
    }

    Ok(())
}
