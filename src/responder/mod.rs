use z_cognition::{BeliefBase, ReasoningEngine, Rule, UtilityFunction};
use tracing::{info, debug};
use rand;

/// Build the reasoning engine with topic-matching rules
pub fn build_reasoning_engine() -> ReasoningEngine {
    let mut engine = ReasoningEngine::new();

    // BDI — specific terms first
    engine.add_rule(
        Rule::new("topic:bdi")
            .with_condition("bdi")
            .with_condition("belief")
            .with_condition("desire")
            .with_condition("intention")
            .with_conclusion("topic:bdi"),
    );

    // Auctions / Market
    engine.add_rule(
        Rule::new("topic:auctions")
            .with_condition("auction")
            .with_condition("market")
            .with_condition("bid")
            .with_condition("english")
            .with_condition("dutch")
            .with_condition("vickrey")
            .with_conclusion("topic:auctions"),
    );

    // Swarm
    engine.add_rule(
        Rule::new("topic:swarm")
            .with_condition("swarm")
            .with_condition("flock")
            .with_condition("consensus")
            .with_condition("drone")
            .with_condition("foraging")
            .with_conclusion("topic:swarm"),
    );

    // Patterns (broad)
    engine.add_rule(
        Rule::new("topic:patterns")
            .with_condition("pattern")
            .with_condition("hierarchy")
            .with_condition("coalition")
            .with_condition("federation")
            .with_condition("team")
            .with_condition("holarchy")
            .with_condition("blackboard")
            .with_condition("organization")
            .with_conclusion("topic:patterns"),
    );

    // Messaging
    engine.add_rule(
        Rule::new("topic:messaging")
            .with_condition("message")
            .with_condition("messaging")
            .with_condition("router")
            .with_condition("performative")
            .with_condition("fipa")
            .with_condition("communicate")
            .with_condition("communication")
            .with_conclusion("topic:messaging"),
    );

    // Cognition
    engine.add_rule(
        Rule::new("topic:cognition")
            .with_condition("cognition")
            .with_condition("reasoning")
            .with_condition("planning")
            .with_condition("utility")
            .with_condition("thinking")
            .with_condition("decision")
            .with_condition("intelligence")
            .with_conclusion("topic:cognition"),
    );

    // Runtime
    engine.add_rule(
        Rule::new("topic:runtime")
            .with_condition("runtime")
            .with_condition("supervisor")
            .with_condition("circuit")
            .with_condition("metric")
            .with_condition("scheduler")
            .with_condition("health")
            .with_condition("restart")
            .with_conclusion("topic:runtime"),
    );

    // Getting started
    engine.add_rule(
        Rule::new("topic:getting_started")
            .with_condition("start")
            .with_condition("install")
            .with_condition("setup")
            .with_condition("begin")
            .with_condition("tutorial")
            .with_condition("beginner")
            .with_condition("learn")
            .with_conclusion("topic:getting_started"),
    );

    // Why Rust
    engine.add_rule(
        Rule::new("topic:why_rust")
            .with_condition("rust")
            .with_condition("performance")
            .with_condition("safe")
            .with_condition("safety")
            .with_condition("fast")
            .with_condition("speed")
            .with_conclusion("topic:why_rust"),
    );

    // Examples
    engine.add_rule(
        Rule::new("topic:examples")
            .with_condition("example")
            .with_condition("demo")
            .with_condition("sample")
            .with_condition("code")
            .with_condition("show")
            .with_conclusion("topic:examples"),
    );

    // Solana
    engine.add_rule(
        Rule::new("topic:solana")
            .with_condition("solana")
            .with_condition("sol")
            .with_condition("defi")
            .with_condition("onchain")
            .with_condition("blockchain")
            .with_condition("swap")
            .with_condition("liquidity")
            .with_conclusion("topic:solana"),
    );

    // Tokenomics (staking, governance, value accrual — more specific than generic token)
    engine.add_rule(
        Rule::new("topic:tokenomics")
            .with_condition("staking")
            .with_condition("stake")
            .with_condition("governance")
            .with_condition("dao")
            .with_condition("tokenomics")
            .with_condition("fee")
            .with_condition("burn")
            .with_condition("holder")
            .with_condition("holders")
            .with_condition("value")
            .with_condition("benefit")
            .with_condition("revenue")
            .with_condition("sharing")
            .with_conclusion("topic:tokenomics"),
    );

    // Token (CA, ticker, buy)
    engine.add_rule(
        Rule::new("topic:token")
            .with_condition("token")
            .with_condition("ticker")
            .with_condition("ca")
            .with_condition("contract")
            .with_condition("pumpfun")
            .with_condition("pump")
            .with_condition("0icai")
            .with_condition("buy")
            .with_conclusion("topic:token"),
    );

    // Owner / creator
    engine.add_rule(
        Rule::new("topic:owner")
            .with_condition("owner")
            .with_condition("creator")
            .with_condition("founder")
            .with_condition("who")
            .with_condition("made")
            .with_condition("built")
            .with_condition("created")
            .with_conclusion("topic:owner"),
    );

    // What is ZeroicAI (broad catch-all)
    engine.add_rule(
        Rule::new("topic:what_is")
            .with_condition("what")
            .with_condition("who")
            .with_condition("about")
            .with_condition("zeroicai")
            .with_condition("explain")
            .with_condition("tell")
            .with_conclusion("topic:what_is"),
    );

    engine
}

/// Extract words from mention text as facts for the engine
fn extract_facts(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|w| !w.starts_with('@'))
        .map(|w| {
            w.to_lowercase()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string()
        })
        .filter(|w| !w.is_empty() && w.len() > 1)
        .collect()
}

/// Get response candidates from beliefs based on inferred topic
fn get_response_candidates(topic: &str, beliefs: &BeliefBase) -> Vec<String> {
    match topic {
        "topic:what_is" => vec![
            format!("{}\n\nLearn more: https://zeroicai.xyz", lookup(beliefs, "what_is_zeroicai")),
            format!("{} {}", lookup(beliefs, "what_is_zeroicai"), lookup(beliefs, "modular")),
        ],
        "topic:patterns" => vec![
            lookup(beliefs, "patterns"),
            format!("{}\n\nExplore: https://github.com/zeroicai/z-examples", lookup(beliefs, "patterns")),
        ],
        "topic:messaging" => vec![
            lookup(beliefs, "messaging"),
            format!("{}\n\n{}", lookup(beliefs, "messaging"), lookup(beliefs, "performatives")),
        ],
        "topic:cognition" => vec![
            format!("{}\n\n{}", lookup(beliefs, "cognition_crate"), lookup(beliefs, "utility")),
            lookup(beliefs, "cognition_crate"),
        ],
        "topic:bdi" => vec![
            lookup(beliefs, "bdi"),
            format!("{}\n\n{}", lookup(beliefs, "bdi"), lookup(beliefs, "beliefs")),
        ],
        "topic:auctions" => vec![
            lookup(beliefs, "market"),
            format!("{}\n\nSee the market_auction example for a full demo.", lookup(beliefs, "market")),
        ],
        "topic:swarm" => vec![
            lookup(beliefs, "swarm"),
            format!("{}\n\n{}", lookup(beliefs, "swarm"), lookup(beliefs, "patterns")),
        ],
        "topic:runtime" => vec![
            format!("{}\n\n{}", lookup(beliefs, "runtime_crate"), lookup(beliefs, "supervisor")),
            format!("{}\n\n{}", lookup(beliefs, "circuit_breaker"), lookup(beliefs, "metrics")),
        ],
        "topic:getting_started" => vec![
            format!("{}\n\n{}\n\n{}", lookup(beliefs, "install"), lookup(beliefs, "examples"), lookup(beliefs, "docs")),
            format!("{}\n\nCheck out our examples: https://github.com/zeroicai/z-examples", lookup(beliefs, "install")),
        ],
        "topic:why_rust" => vec![
            lookup(beliefs, "why_rust"),
            format!("{}\n\n{}", lookup(beliefs, "why_rust"), lookup(beliefs, "design")),
        ],
        "topic:examples" => vec![
            lookup(beliefs, "examples"),
            format!("{}\n\nCovers all 5 crates end-to-end.", lookup(beliefs, "examples")),
        ],
        "topic:tokenomics" => vec![
            lookup(beliefs, "token_tokenomics"),
            format!("{}\n\nCA: yi66MMYBeHvMAbiboJeqVcjXh3b746D3P6nCRfypump", lookup(beliefs, "token_tokenomics")),
        ],
        "topic:token" => vec![
            lookup(beliefs, "token"),
            format!("{}\n\n{}", lookup(beliefs, "token"), lookup(beliefs, "solana")),
        ],
        "topic:owner" => vec![
            lookup(beliefs, "owner"),
            format!("{}\n\nFollow the project: https://github.com/ZeroicAI", lookup(beliefs, "owner")),
        ],
        "topic:solana" => vec![
            lookup(beliefs, "solana"),
            format!("{}\n\n{}", lookup(beliefs, "solana"), lookup(beliefs, "solana_usecase")),
            lookup(beliefs, "defi_agents"),
        ],
        _ => vec![
            format!("{}\n\nAsk me about patterns, messaging, cognition, runtime, or getting started!", lookup(beliefs, "what_is_zeroicai")),
            "I'm ZeroicAI — a multi-agent framework for Rust! Ask me about our 8 patterns, BDI cognition, message routing, or how to get started.\n\nhttps://zeroicai.xyz".to_string(),
        ],
    }
}

/// Look up a belief value, with fallback
fn lookup(beliefs: &BeliefBase, key: &str) -> String {
    let key_owned = key.to_string();
    let results = beliefs.query(move |b: &z_cognition::Belief| b.key() == key_owned);
    results
        .first()
        .map(|b| b.value().to_string())
        .unwrap_or_else(|| format!("(no info on '{}')", key))
}

/// Score response candidates and pick the best one that fits in 280 chars
fn select_best_response(candidates: Vec<String>) -> Option<String> {
    let fit_scorer = UtilityFunction::new("tweet_fit", |state: &[String]| {
        if let Some(text) = state.first() {
            let len = text.len();
            if len > 280 {
                return 0.0;
            }
            if len > 250 {
                return 0.3;
            }
            if len > 150 {
                return 0.8;
            }
            if len > 50 {
                return 1.0;
            }
            0.5
        } else {
            0.0
        }
    });

    let mut best: Option<(f64, String)> = None;

    for candidate in candidates {
        let state = vec![candidate.clone()];
        let score = fit_scorer.evaluate(&state);

        debug!("Candidate ({} chars, score {:.2}): {}...",
            candidate.len(),
            score,
            &candidate[..candidate.len().min(50)]
        );

        if score > 0.0 {
            if let Some((best_score, _)) = &best {
                if score > *best_score {
                    best = Some((score, candidate));
                }
            } else {
                best = Some((score, candidate));
            }
        }
    }

    best.map(|(_, text)| text)
}

/// Truncate response to fit tweet limit
fn truncate_to_tweet(text: String) -> String {
    if text.len() <= 280 {
        return text;
    }
    let truncated = &text[..277];
    if let Some(last_space) = truncated.rfind(' ') {
        format!("{}...", &text[..last_space])
    } else {
        format!("{}...", &text[..277])
    }
}

/// Main entry point: given a mention text, generate a response using ReasoningEngine
pub fn generate_response(
    mention_text: &str,
    beliefs: &BeliefBase,
    engine: &ReasoningEngine,
) -> Option<String> {
    let facts = extract_facts(mention_text);

    if facts.is_empty() {
        info!("No facts extracted from mention, using default response");
        let candidates = get_response_candidates("unknown", beliefs);
        return select_best_response(candidates).map(truncate_to_tweet).map(|body| {
            let agents = ["ZERO", "AXIOM", "NEXUS", "CIPHER", "VECTOR", "NOVA", "FLUX", "DELTA", "ECHO", "PRISM", "FORGE", "SIGMA", "HELIX", "PHANTOM", "APEX"];
            let name = agents[rand::random::<usize>() % agents.len()];
            let with_sig = format!("{}\n\n↳ Agent {}", body, name);
            if with_sig.len() <= 280 { with_sig } else { body }
        });
    }

    info!("Extracted facts: {:?}", facts);

    // Use ReasoningEngine to infer the best topic
    let topic = match engine.best_match(&facts) {
        Some(inference) => {
            info!(
                "Engine matched rule '{}' with {:.0}% confidence → {:?}",
                inference.rule_name,
                inference.confidence * 100.0,
                inference.conclusions
            );
            inference
                .conclusions
                .first()
                .cloned()
                .unwrap_or_else(|| "unknown".to_string())
        }
        None => {
            info!("No rules matched, using default topic");
            "unknown".to_string()
        }
    };

    let candidates = get_response_candidates(&topic, beliefs);
    select_best_response(candidates).map(truncate_to_tweet).map(|body| {
        let agents = ["ZERO", "AXIOM", "NEXUS", "CIPHER", "VECTOR", "NOVA", "FLUX", "DELTA", "ECHO", "PRISM", "FORGE", "SIGMA", "HELIX", "PHANTOM", "APEX"];
        let name = agents[rand::random::<usize>() % agents.len()];
        let with_sig = format!("{}\n\n↳ Agent {}", body, name);
        if with_sig.len() <= 280 { with_sig } else { body }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::build_knowledge_base;

    fn setup() -> (BeliefBase, ReasoningEngine) {
        (build_knowledge_base(), build_reasoning_engine())
    }

    #[test]
    fn test_extract_facts() {
        let facts = extract_facts("@zeroicai what patterns do you support?");
        assert!(facts.contains(&"patterns".to_string()));
        assert!(facts.contains(&"support".to_string()));
        assert!(!facts.iter().any(|f| f.starts_with('@')));
    }

    #[test]
    fn test_engine_matches_patterns() {
        let (beliefs, engine) = setup();
        let response = generate_response("@zeroicai what patterns do you support?", &beliefs, &engine);
        assert!(response.is_some());
        let text = response.unwrap();
        assert!(text.len() <= 280);
        assert!(text.to_lowercase().contains("pattern"));
    }

    #[test]
    fn test_engine_matches_bdi() {
        let (beliefs, engine) = setup();
        let response = generate_response("@zeroicai explain BDI belief desire intention", &beliefs, &engine);
        assert!(response.is_some());
        let text = response.unwrap();
        assert!(text.len() <= 280);
        assert!(text.to_lowercase().contains("belief") || text.to_lowercase().contains("bdi"));
    }

    #[test]
    fn test_engine_matches_swarm() {
        let (beliefs, engine) = setup();
        let response = generate_response("@zeroicai how does the swarm work?", &beliefs, &engine);
        assert!(response.is_some());
        let text = response.unwrap();
        assert!(text.len() <= 280);
    }

    #[test]
    fn test_unknown_gives_default() {
        let (beliefs, engine) = setup();
        let response = generate_response("@zeroicai xyzzy blorp", &beliefs, &engine);
        assert!(response.is_some());
        assert!(response.unwrap().len() <= 280);
    }

    #[test]
    fn test_all_topics_produce_responses() {
        let (beliefs, engine) = setup();
        let queries = vec![
            "@zeroicai what is zeroicai?",
            "@zeroicai what patterns?",
            "@zeroicai how does messaging work?",
            "@zeroicai tell me about cognition",
            "@zeroicai what about BDI?",
            "@zeroicai auction system?",
            "@zeroicai swarm behavior?",
            "@zeroicai runtime supervisor?",
            "@zeroicai how to get started?",
            "@zeroicai why Rust?",
            "@zeroicai show examples",
            "@zeroicai random gibberish xyz",
        ];

        for query in queries {
            let response = generate_response(query, &beliefs, &engine);
            assert!(response.is_some(), "No response for: {}", query);
            assert!(response.unwrap().len() <= 280, "Too long for: {}", query);
        }
    }
}
