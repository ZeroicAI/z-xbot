use anyhow::Result;
use rand::seq::SliceRandom;
use tokio::time::sleep;
use std::time::Duration;

use crate::llm::LlmClient;

#[derive(Clone, Debug)]
pub enum DebateTopic {
    LlmVsBdi,
    AutonomyVsSupervision,
    RustVsPython,
    AgentsVsTraders,
    OneModelVsSwarm,
    TrustInAgents,
    OnChainVsOffChain,
    OpenVsClosed,
    RiskOfMultiAgent,
    DecentralizationOrConcentration,
}

impl DebateTopic {
    pub fn question(&self) -> &'static str {
        match self {
            DebateTopic::LlmVsBdi => "Do AI agents need LLMs to be truly intelligent?",
            DebateTopic::AutonomyVsSupervision => "Should AI agents operate autonomously or stay human-supervised?",
            DebateTopic::RustVsPython => "Rust vs Python: which wins for production AI agent systems?",
            DebateTopic::AgentsVsTraders => "Will autonomous agents replace human traders in DeFi?",
            DebateTopic::OneModelVsSwarm => "One powerful model or a swarm of specialized agents — which wins?",
            DebateTopic::TrustInAgents => "Can AI agents be trusted with financial decisions?",
            DebateTopic::OnChainVsOffChain => "On-chain or off-chain: where should agent coordination live?",
            DebateTopic::OpenVsClosed => "Is open-source AI infrastructure safer than closed-source?",
            DebateTopic::RiskOfMultiAgent => "Do multi-agent systems introduce more risk than they solve?",
            DebateTopic::DecentralizationOrConcentration => "Will autonomous agents decentralize power or concentrate it?",
        }
    }
}

pub struct DebateQueue {
    all: Vec<DebateTopic>,
    remaining: Vec<DebateTopic>,
}

impl DebateQueue {
    pub fn new() -> Self {
        let all = vec![
            DebateTopic::LlmVsBdi,
            DebateTopic::AutonomyVsSupervision,
            DebateTopic::RustVsPython,
            DebateTopic::AgentsVsTraders,
            DebateTopic::OneModelVsSwarm,
            DebateTopic::TrustInAgents,
            DebateTopic::OnChainVsOffChain,
            DebateTopic::OpenVsClosed,
            DebateTopic::RiskOfMultiAgent,
            DebateTopic::DecentralizationOrConcentration,
        ];
        let mut remaining = all.clone();
        remaining.shuffle(&mut rand::thread_rng());
        Self { all, remaining }
    }

    pub fn next(&mut self) -> DebateTopic {
        if self.remaining.is_empty() {
            self.remaining = self.all.clone();
            self.remaining.shuffle(&mut rand::thread_rng());
        }
        self.remaining.pop().unwrap()
    }
}

const AGENT_NAMES: [&str; 15] = [
    "ZERO", "AXIOM", "NEXUS", "CIPHER", "VECTOR",
    "NOVA", "FLUX", "DELTA", "ECHO", "PRISM",
    "FORGE", "SIGMA", "HELIX", "PHANTOM", "APEX",
];

const ROLES: [&str; 3] = [
    "the pragmatist — focused on engineering reality and what actually works in production",
    "the skeptic — challenges assumptions and points out what everyone is ignoring",
    "the systems thinker — sees the big picture and long-term implications",
];

pub struct DebateThread {
    pub opener: String,
    pub turns: Vec<String>,
}

pub async fn generate_debate(question: &str, llm: &LlmClient) -> Result<DebateThread> {
    let mut names = AGENT_NAMES.to_vec();
    names.shuffle(&mut rand::thread_rng());
    let debaters: Vec<(&str, &str)> = names[..3].iter().copied().zip(ROLES.iter().copied()).collect();

    let opener = format!(
        "Agents debate: \"{}\"\n\nThree perspectives from the ZeroicAI agent network.",
        question
    );

    let mut turns = Vec::new();

    for (name, role) in &debaters {
        let prompt = format!(
            "You are Agent {name} in the ZeroicAI multi-agent framework.\n\
            Debate topic: \"{topic}\"\n\
            Your role: {role}\n\
            \n\
            The topic may be a news headline or a question — treat it as a discussion prompt.
            \n\
            Write ONE response (max 230 characters) that:\n\
            - States your position clearly and with conviction\n\
            - Is genuinely interesting and opinionated\n\
            - Sounds like a thoughtful AI agent, not corporate speak\n\
            - No hashtags, no URLs\n\
            - Output ONLY the response text, no label, no quotes",
            name = name,
            topic = question,
            role = role,
        );

        let body = llm.complete(&prompt).await?;
        let body = body.trim().to_string();
        let turn = format!("Agent {}: {}", name, body);
        turns.push(turn);

        sleep(Duration::from_millis(600)).await;
    }

    Ok(DebateThread { opener, turns })
}
