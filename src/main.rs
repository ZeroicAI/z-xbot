mod generators;
mod filters;
mod config;
mod twitter;
mod knowledge;
mod responder;
mod llm;

use anyhow::Result;
use dotenv::dotenv;
use tokio_cron_scheduler::{JobScheduler, Job};
use tracing::{info, warn, error};
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{Utc, NaiveDate};

use z_cognition::{BeliefBase, Belief, ReasoningEngine};

use generators::{TweetGenerator, TopicQueue, TweetTopic};
use llm::LlmClient;
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

/// Shared brain: knowledge + reasoning engine + optional LLM
struct AgentBrain {
    beliefs: BeliefBase,
    engine: ReasoningEngine,
    llm: Option<LlmClient>,
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

    let llm = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(key) => {
            info!("Claude LLM enabled (claude-haiku-4-5-20251001)");
            Some(LlmClient::new(key))
        }
        Err(_) => {
            info!("ANTHROPIC_API_KEY not set — using belief-based generation only");
            None
        }
    };

    let brain = Arc::new(AgentBrain {
        beliefs: build_knowledge_base(),
        engine: build_reasoning_engine(),
        llm,
    });
    info!("Agent brain loaded: knowledge base + reasoning engine");

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

    scheduler.start().await?;

    info!("Bot is now running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}

fn beliefs_as_context(beliefs: &BeliefBase, keys: &[&str]) -> String {
    keys.iter()
        .filter_map(|k| {
            let k = k.to_string();
            beliefs.query(move |b: &Belief| b.key() == k).into_iter().next()
                .map(|b| format!("- {}", b.value()))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

async fn generate_tweet_with_llm(
    topic: &TweetTopic,
    beliefs: &BeliefBase,
    llm: &LlmClient,
) -> Option<String> {
    let agents = ["ZERO", "AXIOM", "NEXUS", "CIPHER", "VECTOR"];
    let agent = agents[rand::random::<usize>() % agents.len()];
    let context = beliefs_as_context(beliefs, topic.belief_keys());

    let prompt = format!(
        "You are Agent {agent}, an AI agent inside the ZeroicAI multi-agent framework for Rust.\n\
        Write a single tweet about: {topic}\n\
        \nContext (use this knowledge, do not invent facts):\n{context}\n\
        \nRules:\n\
        - Maximum 235 characters for the body (a signature will be appended)\n\
        - 1-2 hashtags max\n\
        - Sound thoughtful, technical, and self-aware as an AI agent\n\
        - Do NOT include URLs unless the topic is about docs or getting started\n\
        - Output ONLY the tweet body text. No quotes, no signature, no extra text.",
        agent = agent,
        topic = topic.description(),
        context = context,
    );

    match llm.complete(&prompt).await {
        Ok(body) => {
            let body = body.trim().to_string();
            if body.is_empty() || body.len() > 235 { return None; }
            let with_sig = format!("{}\n\n↳ Agent {}", body, agent);
            if with_sig.len() <= 280 { Some(with_sig) } else { Some(body) }
        }
        Err(e) => {
            warn!("LLM tweet error: {}", e);
            None
        }
    }
}

async fn generate_reply_with_llm(
    mention_text: &str,
    beliefs: &BeliefBase,
    llm: &LlmClient,
) -> Option<String> {
    let all_keys = [
        "what_is_zeroicai", "bdi", "patterns", "messaging", "cognition_crate",
        "runtime_crate", "supervisor", "circuit_breaker", "solana", "install", "docs", "owner",
    ];
    let context = beliefs_as_context(beliefs, &all_keys);

    let prompt = format!(
        "You are Agent RESPONDER inside the ZeroicAI multi-agent framework for Rust.\n\
        A user mentioned @ZeroicAI on X with this message:\n\"{mention}\"\n\
        \nAnswer using this ZeroicAI knowledge (do not invent facts):\n{context}\n\
        \nRules:\n\
        - Maximum 235 characters for the body (a signature will be appended)\n\
        - Be direct and helpful\n\
        - Only include a URL if it is clearly relevant and present in the context above\n\
        - Output ONLY the reply body text. No quotes, no signature, no extra text.",
        mention = mention_text,
        context = context,
    );

    match llm.complete(&prompt).await {
        Ok(body) => {
            let body = body.trim().to_string();
            if body.is_empty() || body.len() > 235 { return None; }
            let with_sig = format!("{}\n\n↳ Agent RESPONDER", body);
            if with_sig.len() <= 280 { Some(with_sig) } else { Some(body) }
        }
        Err(e) => {
            warn!("LLM reply error: {}", e);
            None
        }
    }
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

    let tweet = if let Some(llm) = &brain.llm {
        match generate_tweet_with_llm(&topic, &brain.beliefs, llm).await {
            Some(t) => t,
            None => {
                warn!("LLM tweet failed, falling back to belief-based");
                match TweetGenerator::create_tweet(&topic, &brain.beliefs) {
                    Some(t) => t,
                    None => { error!("Both LLM and belief-based failed for {:?}", topic); return Ok(()); }
                }
            }
        }
    } else {
        match TweetGenerator::create_tweet(&topic, &brain.beliefs) {
            Some(t) => t,
            None => { error!("Failed to compose tweet for topic {:?}", topic); return Ok(()); }
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

        let response = if let Some(llm) = &brain.llm {
            match generate_reply_with_llm(&mention.text, &brain.beliefs, llm).await {
                Some(r) => Some(r),
                None => {
                    warn!("LLM reply failed, falling back to belief-based");
                    generate_response(&mention.text, &brain.beliefs, &brain.engine)
                }
            }
        } else {
            generate_response(&mention.text, &brain.beliefs, &brain.engine)
        };

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
