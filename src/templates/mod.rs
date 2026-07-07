use rand::seq::SliceRandom;

pub struct TweetTemplates;

// All templates must be ≤250 chars to leave room for the ~30 char signature.
// The generator will skip the signature if needed, but we aim to always fit.

impl TweetTemplates {
    /// AI-focused posts
    pub fn ai_templates() -> Vec<&'static str> {
        vec![
            "AI agents are evolving faster than most realize. The future is autonomous systems working together.\n\n#AI #Agents #MachineLearning",
            "The real revolution isn't chatbots — it's multi-agent systems. Agents coordinating decisions and action.\n\n#AI #MultiAgent",
            "Neural networks were just the beginning. Agent swarms are the endgame.\n\n#AI #SwarmIntelligence",
            "AGI won't be one model. It'll be thousands of specialized agents in perfect coordination.\n\n#AGI #Agents",
            "While everyone's playing with prompts, serious builders are shipping autonomous agent systems.\n\n#AI #Automation",
            "BDI architecture: agents that hold Beliefs, form Desires, commit to Intentions. Not a chatbot. A mind.\n\n#AI #Agents #BDI",
            "The shift from LLM wrappers to true agent systems is the most underrated transition in AI right now.\n\n#AI #Agents",
            "Autonomous agents don't just respond — they plan, reason, and act. That gap is everything.\n\n#AI #Agents",
        ]
    }

    /// ZeroicAI-specific posts
    pub fn zeroicai_templates() -> Vec<&'static str> {
        vec![
            "Production-ready multi-agent systems in Rust. BDI architecture, swarm coordination, fault tolerance — batteries included.\n\n#Rust #ZeroicAI",
            "Your agents deserve Rust's safety and performance. No GC pauses. No Python spaghetti. Just speed.\n\n#Rust #ZeroicAI",
            "8 org patterns: Hierarchy, Swarm, Market, Coalition, Team, Holarchy, Federation, Blackboard. All in ZeroicAI.\n\n#ZeroicAI #Rust",
            "FIPA messaging. BDI cognition. Fault-tolerant runtime. Swarm intelligence. All open source.\n\n#Rust #ZeroicAI #Agents",
            "ZeroicAI agents coordinate like a team, reason like individuals, and scale like infrastructure.\n\n#ZeroicAI #MultiAgent",
            "Most agent frameworks are demos. ZeroicAI is built for production — Rust runtime, typed messaging, supervised recovery.\n\n#ZeroicAI #Rust",
            "z-cognition. z-messaging. z-patterns. z-runtime. Every layer of multi-agent intelligence, composable.\n\n#ZeroicAI #Rust",
        ]
    }

    /// Crypto + AI hybrid posts
    pub fn crypto_ai_templates() -> Vec<&'static str> {
        vec![
            "Blockchain + AI agents: on-chain coordination, autonomous execution, trustless cooperation. DeFi is going agentic.\n\n#DeFi #AIAgents",
            "Smart contracts are logic. AI agents are judgment. Combine them and you get autonomous financial systems.\n\n#AI #Blockchain #DeFi",
            "MEV but it's AI agent swarms competing in microseconds. That's where this is heading.\n\n#MEV #AIAgents #Crypto",
            "Every major protocol will have AI agents soon. The ones sleeping on this will regret it.\n\n#DeFi #AI #Agents",
            "Algo trading → AI trading agents → Agent swarms coordinating trades. We're entering the swarm era.\n\n#Crypto #AIAgents",
            "Autonomous agents that negotiate, trade, and settle on-chain without human approval. That's not sci-fi anymore.\n\n#DeFi #AI #Solana",
        ]
    }

    /// Meme coin + AI posts
    pub fn meme_ai_templates() -> Vec<&'static str> {
        vec![
            "AI agent tokens are the new meta. Utility + narrative = unstoppable.\n\n#AI #Crypto",
            "Doge had a dog. We have autonomous reasoning agents. Different era, same energy.\n\n#AI #Crypto",
            "AI tokens aren't just memes. They're infrastructure for autonomous economies. (Also they're memes.)\n\n#AI #Crypto",
            "The best performing asset next cycle will be an AI agent token nobody's heard of yet. Screenshot this.\n\n#Crypto #AI",
            "Agents that generate alpha, execute trades, and post their own updates. The loop is closing.\n\n#AI #Crypto #Agents",
        ]
    }

    /// General posts
    pub fn general_bull_templates() -> Vec<&'static str> {
        vec![
            "Agent economies are coming. Agents trading, coordinating value, building wealth autonomously.\n\n#AI #Agents #Future",
            "Going from single AI models to multi-agent systems is like going from single-player to MMO. We're going multiplayer.\n\n#AI #Agents",
            "Your next coworker won't be human. It'll be a swarm of specialized AI agents. Get ready.\n\n#AI #FutureOfWork",
            "AI agents don't sleep, don't take breaks, and scale infinitely. The workforce shift is already here.\n\n#AI #Automation",
            "Building AI agents right now is like building websites in 1995. Early. Weird. Massively underpriced.\n\n#AI #Agents #Tech",
            "Self-healing systems, decentralized AI markets, autonomous governance. All of this runs on agent infrastructure.\n\n#AI #Agents",
            "The organizations that win the next decade will be the ones that figured out how to deploy agent teams.\n\n#AI #Automation #Future",
        ]
    }

    pub fn random_ai_tweet() -> String {
        Self::ai_templates()
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string()
    }

    pub fn random_zeroicai_tweet() -> String {
        Self::zeroicai_templates()
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string()
    }

    pub fn random_crypto_tweet() -> String {
        Self::crypto_ai_templates()
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string()
    }

    pub fn random_meme_tweet() -> String {
        Self::meme_ai_templates()
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string()
    }

    pub fn random_bull_tweet() -> String {
        Self::general_bull_templates()
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify every template fits within 250 chars (leaves room for signature)
    #[test]
    fn test_all_templates_under_limit() {
        let max_len = 250;
        let all_templates: Vec<(&str, Vec<&str>)> = vec![
            ("ai", TweetTemplates::ai_templates()),
            ("zeroicai", TweetTemplates::zeroicai_templates()),
            ("crypto", TweetTemplates::crypto_ai_templates()),
            ("meme", TweetTemplates::meme_ai_templates()),
            ("general", TweetTemplates::general_bull_templates()),
        ];

        for (category, templates) in all_templates {
            for (i, template) in templates.iter().enumerate() {
                assert!(
                    template.len() <= max_len,
                    "Template {}[{}] is {} chars (max {}): {:?}",
                    category,
                    i,
                    template.len(),
                    max_len,
                    &template[..50.min(template.len())]
                );
            }
        }
    }
}