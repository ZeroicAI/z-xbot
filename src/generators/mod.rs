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
                format!("{}\n\nzeroicai.xyz\n\n#ZeroicAI #Rust #Agents", l("what_is_zeroicai")),
                format!("{}\n\n{}\n\n#ZeroicAI #Rust", l("what_is_zeroicai"), l("modular")),
            ],
            TweetTopic::WhyRust => vec![
                format!("{}\n\n#Rust #AI #Performance", l("why_rust")),
                format!("{}\n\n{}\n\n#Rust #ZeroicAI", l("why_rust"), l("design")),
            ],
            TweetTopic::BDI => vec![
                format!("{}\n\n#AI #BDI #Agents", l("bdi")),
                format!("Most AI systems react. BDI agents reason.\n\n{}\n\n#AI #BDI", l("bdi")),
            ],
            TweetTopic::Messaging => vec![
                format!("{}\n\n#AI #Agents #FIPA", l("messaging")),
                format!("{}\n\n{}\n\n#ZeroicAI", l("messaging"), l("performatives")),
            ],
            TweetTopic::SwarmPattern => vec![
                format!("{}\n\n#AI #Swarm #ZeroicAI", l("swarm")),
                format!(
                    "No central controller. Agents follow local rules. The swarm emerges.\n\n{}\n\n#ZeroicAI",
                    l("swarm")
                ),
            ],
            TweetTopic::MarketPattern => vec![
                format!("{}\n\n#AI #Agents #ZeroicAI", l("market")),
                format!("Let agents bid for resources.\n\n{}\n\n#ZeroicAI #Agents", l("market")),
            ],
            TweetTopic::CoalitionPattern => vec![
                format!("{}\n\n#ZeroicAI #MultiAgent", l("coalition")),
                format!(
                    "Temporary alliances. Shared goals. Clean dissolve.\n\n{}\n\n#ZeroicAI",
                    l("coalition")
                ),
            ],
            TweetTopic::RuntimeSupervisor => vec![
                format!("{}\n\n{}\n\n#ZeroicAI #Rust", l("runtime_crate"), l("supervisor")),
                format!("Agents fail. Systems shouldn't.\n\n{}\n\n#ZeroicAI", l("supervisor")),
            ],
            TweetTopic::CircuitBreaker => vec![
                format!("{}\n\n#ZeroicAI #Reliability #Rust", l("circuit_breaker")),
                format!(
                    "Cascading failures kill distributed systems.\n\n{}\n\n#ZeroicAI",
                    l("circuit_breaker")
                ),
            ],
            TweetTopic::OrgPatterns => vec![
                format!("{}\n\n#ZeroicAI #MultiAgent #Rust", l("patterns")),
                format!(
                    "8 ways to organize agents. One framework.\n\n{}\n\n#ZeroicAI",
                    l("patterns")
                ),
            ],
            TweetTopic::CrateOverview => vec![
                format!("{}\n\n#ZeroicAI #Rust #OpenSource", l("crates")),
                format!("{}\n\n{}\n\n#ZeroicAI #Rust", l("crates"), l("modular")),
            ],
            TweetTopic::Solana => vec![
                format!("{}\n\n#ZeroicAI #Solana #AI", l("solana")),
                format!("{}\n\n#Solana #DeFi #Agents", l("solana_usecase")),
            ],
            TweetTopic::DeFiAgents => vec![
                format!("{}\n\n#DeFi #AI #Solana", l("defi_agents")),
                format!(
                    "Autonomous DeFi. No human approval needed.\n\n{}\n\n#Solana #ZeroicAI",
                    l("defi_agents")
                ),
            ],
            TweetTopic::GettingStarted => vec![
                format!("{}\n\n{}\n\n#ZeroicAI #Rust", l("install"), l("docs")),
                format!(
                    "Ready to build agents?\n\n{}\n\n{}\n\n#ZeroicAI",
                    l("install"),
                    l("examples")
                ),
            ],
            TweetTopic::OpenSource => vec![
                format!("{}\n\nzeroicai.xyz\n\n#OpenSource #Rust #ZeroicAI", l("license")),
                format!(
                    "Open source. Production ready. No lock-in.\n\n{}\n\n#ZeroicAI",
                    l("license")
                ),
            ],
            TweetTopic::Community => vec![
                format!("{}\n\n#ZeroicAI #AI #Community", l("telegram")),
                format!(
                    "Building the agent economy together.\n\n{}\n\n#ZeroicAI",
                    l("telegram")
                ),
            ],
            TweetTopic::AgentVsScript => vec![
                "An agent that can't recover from failure isn't an agent. It's a script with ambition.\n\n#AI #Agents #Engineering".to_string(),
                "Single agents are toys. Multi-agent systems are infrastructure. The difference is coordination.\n\n#AI #MultiAgent #ZeroicAI".to_string(),
            ],
            TweetTopic::MultiAgentShift => vec![
                "The shift from LLM wrappers to true agent systems is the most underrated transition in AI right now.\n\n#AI #Agents #ZeroicAI".to_string(),
                "Going from one model to a coordinated agent team is like going from single-player to MMO. We're going multiplayer.\n\n#AI #Agents".to_string(),
            ],
            TweetTopic::FaultTolerance => vec![
                format!(
                    "{}\n\n{}\n\n#ZeroicAI #Reliability",
                    l("circuit_breaker"),
                    l("backoff")
                ),
                format!(
                    "Self-healing is not a feature. It's a requirement.\n\n{}\n\n#ZeroicAI #Rust",
                    l("supervisor")
                ),
            ],
            TweetTopic::FipaStandard => vec![
                format!("{}\n\n#FIPA #AI #Agents #ZeroicAI", l("fipa")),
                format!(
                    "ZeroicAI implements FIPA — the IEEE standard for agent communication.\n\n{}\n\n#ZeroicAI",
                    l("fipa")
                ),
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
