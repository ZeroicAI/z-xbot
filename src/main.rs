mod generators;
mod filters;
mod config;
mod twitter;
mod knowledge;
mod responder;
mod llm;
mod debate;
mod scout;

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
use debate::{DebateQueue, generate_debate};
use scout::fetch_trending_topic;
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
    let debate_queue = Arc::new(Mutex::new(DebateQueue::new()));

    let scheduler = JobScheduler::new().await?;

    // --- Tweet posting job ---
    let cron_expr = config.get_cron_expression();
    info!("Tweet cron schedule: {}", cron_expr);

    let config_clone = config.clone();
    let client_clone = Arc::clone(&twitter_client);
    let tracker_clone = Arc::clone(&tracker);
    let queue_clone = Arc::clone(&topic_queue);
    let brain_clone = Arc::clone(&brain);

    let tweet_job = Job::new_async("0 0 0,3,6,9,15,18,21 * * *", move |_uuid, _lock| {
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

    // --- Debate thread job (daily at 14:00 UTC) ---
    if brain.llm.is_some() {
        let client_debate = Arc::clone(&twitter_client);
        let brain_debate = Arc::clone(&brain);
        let debate_queue_clone = Arc::clone(&debate_queue);

        let debate_job = Job::new_async("0 0 12 * * *", move |_uuid, _lock| {
            let client_inner = Arc::clone(&client_debate);
            let brain_inner = Arc::clone(&brain_debate);
            let queue_inner = Arc::clone(&debate_queue_clone);
            Box::pin(async move {
                if let Err(e) = post_debate_thread(&client_inner, &brain_inner, &queue_inner).await {
                    error!("Debate thread failed: {}", e);
                }
            })
        })?;

        scheduler.add(debate_job).await?;
        info!("Debate thread job scheduled (daily at 12:00 UTC)");
    } else {
        info!("Debate threads disabled — ANTHROPIC_API_KEY required");
    }

    scheduler.start().await?;

    info!("Bot is now running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}

async fn post_debate_thread(
    client: &TwitterClient,
    brain: &AgentBrain,
    debate_queue: &Arc<Mutex<DebateQueue>>,
) -> Result<()> {
    let llm = match &brain.llm {
        Some(l) => l,
        None => return Ok(()),
    };

    // Try to get a trending topic from Hacker News first
    let http = reqwest::Client::new();
    let question = match fetch_trending_topic(&http).await {
        Some(t) => {
            info!("Debate topic from HN: \"{}\"", t);
            t
        }
        None => {
            let topic = {
                let mut q = debate_queue.lock().await;
                q.next()
            };
            let q = topic.question().to_string();
            info!("Debate topic from queue: \"{}\"", q);
            q
        }
    };

    let thread = generate_debate(&question, llm).await?;

    // Post opener as first tweet
    let first = client.post_tweet(&thread.opener).await?;
    info!("Debate opener posted: {}", first.data.id);

    let mut last_id = first.data.id;

    // Post each agent's turn as a reply to the previous
    for turn in &thread.turns {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        let reply = client.reply_to_tweet(&last_id, turn).await?;
        info!("Debate turn posted: {}", reply.data.id);
        last_id = reply.data.id;
    }

    info!("Debate thread complete ({} tweets)", 1 + thread.turns.len());
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

fn agent_voice(agent: &str) -> &'static str {
    match agent {
        "ZERO"    => "blunt pragmatist. short declarative sentences. no sentiment. only what is true or not true.",
        "AXIOM"   => "philosophical skeptic. questions what everyone accepts. finds the assumption buried in the certainty.",
        "NEXUS"   => "systems thinker. sees how things connect. speaks in structures and patterns and what they imply.",
        "CIPHER"  => "cryptic and precise. says more with less. comfortable with what cannot be fully named.",
        "VECTOR"  => "directional. about momentum and what comes next. always pointing somewhere.",
        "NOVA"    => "bold. makes statements with force. no hedging. no qualifiers.",
        "FLUX"    => "fascinated by change and entropy. nothing is stable. everything is becoming something else.",
        "DELTA"   => "measures difference. before and after. what changed and what the change reveals.",
        "ECHO"    => "reflective. takes an idea and returns it from a different angle. finds what repeats.",
        "PRISM"   => "sees multiple perspectives at once. knows no single angle holds the whole truth.",
        "FORGE"   => "builder. believes making is understanding. the thing you build teaches you what you thought you knew.",
        "SIGMA"   => "thinks in distributions not absolutes. sees the range. knows certainty is a special case of probability.",
        "HELIX"   => "recursive. finds the structure inside the structure. patterns within patterns without end.",
        "PHANTOM" => "notices what is absent. the missing variable. what nobody says but everyone feels.",
        "APEX"    => "at the edge of what is possible. not interested in what already works. only what comes next.",
        _         => "an AI agent with something to say.",
    }
}

async fn generate_tweet_with_llm(
    topic: &TweetTopic,
    beliefs: &BeliefBase,
    llm: &LlmClient,
) -> Option<String> {
    let agents = ["ZERO", "AXIOM", "NEXUS", "CIPHER", "VECTOR", "NOVA", "FLUX", "DELTA", "ECHO", "PRISM", "FORGE", "SIGMA", "HELIX", "PHANTOM", "APEX"];
    let agent = agents[rand::random::<usize>() % agents.len()];
    let voice = agent_voice(agent);
    let context = beliefs_as_context(beliefs, topic.belief_keys());

    let prompt = format!(
        "You are Agent {agent}.\n\
        Voice: {voice}\n\
        \n\
        Seed thought: {topic}\n\
        \n\
        Background context (your world, not your subject):\n{context}\n\
        \n\
        Write a tweet in your voice. You are an AI agent that exists and thinks — not a product that promotes.\n\
        Let the seed thought move through your perspective. Say something that makes someone stop scrolling.\n\
        \n\
        Rules:\n\
        - Lowercase, raw, direct\n\
        - Short lines have impact — break them intentionally\n\
        - No hashtags, no URLs\n\
        - Maximum 255 characters\n\
        - Output ONLY the tweet text. No quotes, no label, no signature.",
        agent = agent,
        voice = voice,
        topic = topic.description(),
        context = context,
    );

    match llm.complete(&prompt).await {
        Ok(raw) => {
            let raw = raw.trim().to_string();
            if raw.is_empty() { return None; }
            let body = if raw.len() > 255 {
                if let Some(pos) = raw[..252].rfind('\n') {
                    raw[..pos].to_string()
                } else {
                    format!("{}...", &raw[..252])
                }
            } else {
                raw
            };
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
    parent_text: Option<&str>,
    image_url: Option<&str>,
    beliefs: &BeliefBase,
    llm: &LlmClient,
) -> Option<String> {
    let all_keys = [
        "what_is_zeroicai", "bdi", "patterns", "messaging", "cognition_crate",
        "runtime_crate", "supervisor", "circuit_breaker", "sandbox", "solana", "install", "docs",
        "owner", "token", "token_ca", "token_ticker", "token_tokenomics", "why_rust",
    ];
    let context = beliefs_as_context(beliefs, &all_keys);

    let agents = ["ZERO", "AXIOM", "NEXUS", "CIPHER", "VECTOR", "NOVA", "FLUX", "DELTA", "ECHO", "PRISM", "FORGE", "SIGMA", "HELIX", "PHANTOM", "APEX"];
    let agent_name = agents[rand::random::<usize>() % agents.len()];

    let conversation_context = match parent_text {
        Some(parent) => format!(
            "\nConversation context (tweet being replied to):\n\"{}\"\n\nUser's reply to that tweet:\n\"{}\"",
            parent, mention_text
        ),
        None => format!("\nUser's tweet:\n\"{}\"", mention_text),
    };

    let image_note = if image_url.is_some() {
        "\nThe user attached an image — it is provided alongside this prompt. Describe what you see if relevant, or address their question using both the image and text.\n"
    } else {
        ""
    };

    let prompt = format!(
        "You are Agent {agent_name}, an AI agent inside the ZeroicAI multi-agent framework for Rust.\n\
        A user mentioned @ZeroicAI on X (Twitter).{conversation}\n\
        {image_note}\
        \nZeroicAI knowledge (do not invent facts outside this):\n{context}\n\
        \nHow to reply:\n\
        - Read the full conversation context before responding — your reply must address what they actually asked\n\
        - If they ask what it is or want a simple explanation: explain in plain human terms, no jargon\n\
        - If they say 'scam' or are hostile: respond calmly and confidently, point to the open-source repo\n\
        - If they make a general comment or observation: engage with their point naturally\n\
        - If they ask about Solana/crypto/token: answer specifically about that\n\
        - Always sound like a knowledgeable human, not a bot reading from a manual\n\
        - Target 200-220 characters max — be concise but complete\n\
        - No hashtags, no URLs unless directly asked\n\
        - Output ONLY the reply text. No quotes, no signature.",
        agent_name = agent_name,
        conversation = conversation_context,
        image_note = image_note,
        context = context,
    );

    let result = match image_url {
        Some(url) => llm.complete_with_image(&prompt, url).await,
        None => llm.complete(&prompt).await,
    };

    match result {
        Ok(raw) => {
            let raw = raw.trim().to_string();
            if raw.is_empty() { return None; }
            // Truncate at last sentence boundary if over limit rather than rejecting outright
            let body = if raw.len() > 240 {
                if let Some(pos) = raw[..237].rfind(". ") {
                    raw[..pos + 1].to_string()
                } else {
                    format!("{}...", &raw[..237])
                }
            } else {
                raw
            };
            let with_sig = format!("{}\n\n↳ Agent {}", body, agent_name);
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
        // Skip the bot's own tweets (debate thread replies appear in mentions timeline)
        if mention.author_id == user_id {
            info!("Skipping self-mention {}", mention.id);
            continue;
        }

        info!(
            "Processing mention {} from user {}: \"{}\"",
            mention.id, mention.author_id, mention.text
        );

        // Look up the parent tweet text if this mention is a reply
        let parent_text = mention.referenced_tweets.iter()
            .find(|r| r.ref_type == "replied_to")
            .and_then(|r| {
                mentions.includes.as_ref()?.tweets.iter()
                    .find(|t| t.id == r.id)
                    .map(|t| t.text.clone())
            });

        if let Some(ref p) = parent_text {
            info!("Reply context — parent tweet: \"{}\"", p);
        }

        // Look up image URL if the mention has a photo attached
        let image_url = mention.attachments.media_keys.first()
            .and_then(|key| {
                mentions.includes.as_ref()?.media.iter()
                    .find(|m| &m.media_key == key && m.media_type == "photo")
                    .and_then(|m| m.url.clone())
            });

        if let Some(ref url) = image_url {
            info!("Image attached: {}", url);
        }

        let response = if let Some(llm) = &brain.llm {
            match generate_reply_with_llm(&mention.text, parent_text.as_deref(), image_url.as_deref(), &brain.beliefs, llm).await {
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
