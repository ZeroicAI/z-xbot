use rand::seq::SliceRandom;
use tracing::warn;
use z_cognition::{BeliefBase, Belief};

const MAX_TWEET_LENGTH: usize = 280;

#[derive(Clone, Debug)]
pub enum TweetTopic {
    WhatIsZeroicAI,
    WhyRust,
    BDI,
    Messaging,
    SwarmPattern,
    MarketPattern,
    CoalitionPattern,
    RuntimeSupervisor,
    CircuitBreaker,
    OrgPatterns,
    CrateOverview,
    Solana,
    DeFiAgents,
    GettingStarted,
    OpenSource,
    Community,
    AgentVsScript,
    MultiAgentShift,
    FaultTolerance,
    FipaStandard,
}

/// Cycles through all topics in shuffled order before repeating any.
/// Guarantees no duplicate posts until the full topic set is exhausted.
pub struct TopicQueue {
    all: Vec<TweetTopic>,
    remaining: Vec<TweetTopic>,
}

impl TopicQueue {
    pub fn new() -> Self {
        let all = vec![
            TweetTopic::WhatIsZeroicAI,
            TweetTopic::WhyRust,
            TweetTopic::BDI,
            TweetTopic::Messaging,
            TweetTopic::SwarmPattern,
            TweetTopic::MarketPattern,
            TweetTopic::CoalitionPattern,
            TweetTopic::RuntimeSupervisor,
            TweetTopic::CircuitBreaker,
            TweetTopic::OrgPatterns,
            TweetTopic::CrateOverview,
            TweetTopic::Solana,
            TweetTopic::DeFiAgents,
            TweetTopic::GettingStarted,
            TweetTopic::OpenSource,
            TweetTopic::Community,
            TweetTopic::AgentVsScript,
            TweetTopic::MultiAgentShift,
            TweetTopic::FaultTolerance,
            TweetTopic::FipaStandard,
        ];
        let mut remaining = all.clone();
        remaining.shuffle(&mut rand::thread_rng());
        Self { all, remaining }
    }

    pub fn next(&mut self) -> TweetTopic {
        if self.remaining.is_empty() {
            self.remaining = self.all.clone();
            self.remaining.shuffle(&mut rand::thread_rng());
        }
        self.remaining.pop().unwrap()
    }
}

impl TweetTopic {
    pub fn description(&self) -> &'static str {
        match self {
            TweetTopic::WhatIsZeroicAI   => "what ZeroicAI is and why it matters",
            TweetTopic::WhyRust          => "why ZeroicAI is built in Rust",
            TweetTopic::BDI              => "BDI (Belief-Desire-Intention) cognitive architecture",
            TweetTopic::Messaging        => "agent messaging and FIPA performatives",
            TweetTopic::SwarmPattern     => "the swarm coordination pattern",
            TweetTopic::MarketPattern    => "the market auction pattern for resource allocation",
            TweetTopic::CoalitionPattern => "the coalition pattern for temporary agent alliances",
            TweetTopic::RuntimeSupervisor => "the runtime supervisor for agent fault tolerance",
            TweetTopic::CircuitBreaker   => "circuit breakers preventing cascading failures",
            TweetTopic::OrgPatterns      => "the 8 organizational patterns for multi-agent systems",
            TweetTopic::CrateOverview    => "the 5 ZeroicAI crates and what each does",
            TweetTopic::Solana           => "ZeroicAI's native Solana blockchain integration",
            TweetTopic::DeFiAgents       => "autonomous DeFi agents on Solana",
            TweetTopic::GettingStarted   => "how to get started with ZeroicAI",
            TweetTopic::OpenSource       => "ZeroicAI being open source under MIT/Apache-2.0",
            TweetTopic::Community        => "the ZeroicAI community on Telegram",
            TweetTopic::AgentVsScript    => "the difference between a true agent and a script",
            TweetTopic::MultiAgentShift  => "the industry shift from single models to multi-agent systems",
            TweetTopic::FaultTolerance   => "self-healing fault tolerance in agent systems",
            TweetTopic::FipaStandard     => "FIPA — the IEEE standard for agent communication",
        }
    }

    pub fn belief_keys(&self) -> &'static [&'static str] {
        match self {
            TweetTopic::WhatIsZeroicAI   => &["what_is_zeroicai", "modular"],
            TweetTopic::WhyRust          => &["why_rust", "design"],
            TweetTopic::BDI              => &["bdi", "beliefs", "utility"],
            TweetTopic::Messaging        => &["messaging", "performatives"],
            TweetTopic::SwarmPattern     => &["swarm"],
            TweetTopic::MarketPattern    => &["market"],
            TweetTopic::CoalitionPattern => &["coalition"],
            TweetTopic::RuntimeSupervisor => &["runtime_crate", "supervisor"],
            TweetTopic::CircuitBreaker   => &["circuit_breaker", "backoff"],
            TweetTopic::OrgPatterns      => &["patterns"],
            TweetTopic::CrateOverview    => &["crates", "modular"],
            TweetTopic::Solana           => &["solana", "solana_usecase"],
            TweetTopic::DeFiAgents       => &["defi_agents", "solana"],
            TweetTopic::GettingStarted   => &["install", "docs", "examples"],
            TweetTopic::OpenSource       => &["license", "github"],
            TweetTopic::Community        => &["telegram", "twitter"],
            TweetTopic::AgentVsScript    => &["bdi", "why_rust"],
            TweetTopic::MultiAgentShift  => &["patterns", "what_is_zeroicai"],
            TweetTopic::FaultTolerance   => &["circuit_breaker", "supervisor", "backoff"],
            TweetTopic::FipaStandard     => &["fipa", "messaging"],
        }
    }
}

pub struct TweetGenerator;

impl TweetGenerator {
    fn lookup(beliefs: &BeliefBase, key: &str) -> String {
        let key_owned = key.to_string();
        let results = beliefs.query(move |b: &Belief| b.key() == key_owned);
        results
            .first()
            .map(|b| b.value().to_string())
            .unwrap_or_default()
    }

    fn agent_name() -> &'static str {
        let agents = ["ZERO", "AXIOM", "NEXUS", "CIPHER", "VECTOR"];
        agents.choose(&mut rand::thread_rng()).unwrap()
    }

    /// Compose tweet candidates for a topic using the belief base.
    /// Returns multiple candidates; the best-fitting one is selected.
    fn compose(topic: &TweetTopic, beliefs: &BeliefBase) -> Vec<String> {
        let l = |k: &str| Self::lookup(beliefs, k);

        match topic {
            TweetTopic::WhatIsZeroicAI => vec![
                format!("{}\n\nzeroicai.xyz", l("what_is_zeroicai")),
                format!("{}\n\n{}", l("what_is_zeroicai"), l("modular")),
            ],
            TweetTopic::WhyRust => vec![
                l("why_rust"),
                format!("{}\n\n{}", l("why_rust"), l("design")),
            ],
            TweetTopic::BDI => vec![
                l("bdi"),
                format!("Most AI systems react. BDI agents reason.\n\n{}", l("bdi")),
            ],
            TweetTopic::Messaging => vec![
                l("messaging"),
                format!("{}\n\n{}", l("messaging"), l("performatives")),
            ],
            TweetTopic::SwarmPattern => vec![
                l("swarm"),
                format!("No central controller. Agents follow local rules. The swarm emerges.\n\n{}", l("swarm")),
            ],
            TweetTopic::MarketPattern => vec![
                l("market"),
                format!("Let agents bid for resources.\n\n{}", l("market")),
            ],
            TweetTopic::CoalitionPattern => vec![
                l("coalition"),
                format!("Temporary alliances. Shared goals. Clean dissolve.\n\n{}", l("coalition")),
            ],
            TweetTopic::RuntimeSupervisor => vec![
                format!("{}\n\n{}", l("runtime_crate"), l("supervisor")),
                format!("Agents fail. Systems shouldn't.\n\n{}", l("supervisor")),
            ],
            TweetTopic::CircuitBreaker => vec![
                l("circuit_breaker"),
                format!("Cascading failures kill distributed systems.\n\n{}", l("circuit_breaker")),
            ],
            TweetTopic::OrgPatterns => vec![
                l("patterns"),
                format!("8 ways to organize agents. One framework.\n\n{}", l("patterns")),
            ],
            TweetTopic::CrateOverview => vec![
                l("crates"),
                format!("{}\n\n{}", l("crates"), l("modular")),
            ],
            TweetTopic::Solana => vec![
                l("solana"),
                l("solana_usecase"),
            ],
            TweetTopic::DeFiAgents => vec![
                l("defi_agents"),
                format!("Autonomous DeFi. No human approval needed.\n\n{}", l("defi_agents")),
            ],
            TweetTopic::GettingStarted => vec![
                format!("{}\n\n{}", l("install"), l("docs")),
                format!("Ready to build agents?\n\n{}\n\n{}", l("install"), l("examples")),
            ],
            TweetTopic::OpenSource => vec![
                format!("{}\n\nzeroicai.xyz", l("license")),
                format!("Open source. Production ready. No lock-in.\n\n{}", l("license")),
            ],
            TweetTopic::Community => vec![
                l("telegram"),
                format!("Building the agent economy together.\n\n{}", l("telegram")),
            ],
            TweetTopic::AgentVsScript => vec![
                "An agent that can't recover from failure isn't an agent. It's a script with ambition.".to_string(),
                "Single agents are toys. Multi-agent systems are infrastructure. The difference is coordination.".to_string(),
            ],
            TweetTopic::MultiAgentShift => vec![
                "The shift from LLM wrappers to true agent systems is the most underrated transition in AI right now.".to_string(),
                "Going from one model to a coordinated agent team is like going from single-player to MMO. We're going multiplayer.".to_string(),
            ],
            TweetTopic::FaultTolerance => vec![
                format!("{}\n\n{}", l("circuit_breaker"), l("backoff")),
                format!("Self-healing is not a feature. It's a requirement.\n\n{}", l("supervisor")),
            ],
            TweetTopic::FipaStandard => vec![
                l("fipa"),
                format!("ZeroicAI implements FIPA — the IEEE standard for agent communication.\n\n{}", l("fipa")),
            ],
        }
    }

    /// Pick the longest candidate that fits within the tweet limit.
    fn pick_best(candidates: Vec<String>) -> Option<String> {
        candidates
            .into_iter()
            .filter(|t| t.len() <= MAX_TWEET_LENGTH)
            .max_by_key(|t| t.len())
    }

    /// Compose a tweet from the belief base for the given topic.
    pub fn create_tweet(topic: &TweetTopic, beliefs: &BeliefBase) -> Option<String> {
        let candidates = Self::compose(topic, beliefs);
        let body = Self::pick_best(candidates)?;

        let agent = Self::agent_name();
        let signature = format!("\n\n↳ Agent {}", agent);
        let with_sig = format!("{}{}", body, signature);

        if with_sig.len() <= MAX_TWEET_LENGTH {
            Some(with_sig)
        } else {
            warn!("Signature skipped — tweet at {} chars", body.len());
            Some(body)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::build_knowledge_base;

    #[test]
    fn test_all_topics_produce_tweet() {
        let beliefs = build_knowledge_base();
        let topics = vec![
            TweetTopic::WhatIsZeroicAI, TweetTopic::WhyRust, TweetTopic::BDI,
            TweetTopic::Messaging, TweetTopic::SwarmPattern, TweetTopic::MarketPattern,
            TweetTopic::CoalitionPattern, TweetTopic::RuntimeSupervisor, TweetTopic::CircuitBreaker,
            TweetTopic::OrgPatterns, TweetTopic::CrateOverview, TweetTopic::Solana,
            TweetTopic::DeFiAgents, TweetTopic::GettingStarted, TweetTopic::OpenSource,
            TweetTopic::Community, TweetTopic::AgentVsScript, TweetTopic::MultiAgentShift,
            TweetTopic::FaultTolerance, TweetTopic::FipaStandard,
        ];
        for topic in &topics {
            let tweet = TweetGenerator::create_tweet(topic, &beliefs);
            assert!(tweet.is_some(), "No tweet for topic {:?}", topic);
            let text = tweet.unwrap();
            assert!(text.len() <= 280, "Tweet too long for topic {:?}: {} chars", topic, text.len());
        }
    }

    #[test]
    fn test_topic_queue_no_immediate_repeat() {
        let mut queue = TopicQueue::new();
        let total = queue.all.len();
        let mut seen = std::collections::HashSet::new();
        for _ in 0..total {
            let topic = format!("{:?}", queue.next());
            assert!(seen.insert(topic.clone()), "Topic repeated before full cycle: {}", topic);
        }
    }

    #[test]
    fn test_topic_queue_resets_after_full_cycle() {
        let mut queue = TopicQueue::new();
        let total = queue.all.len();
        for _ in 0..total * 2 {
            queue.next();
        }
    }
}
