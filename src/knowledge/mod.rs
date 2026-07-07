use z_cognition::{BeliefBase, Belief};

/// Build the ZeroicAI knowledge base with all framework facts
pub fn build_knowledge_base() -> BeliefBase {
    let mut beliefs = BeliefBase::new();

    // Core framework
    beliefs.add(Belief::new("what_is_zeroicai", "ZeroicAI is a modular multi-agent framework for Rust. It provides agent lifecycles, messaging, cognition, organizational patterns, and supervised runtime execution."));
    beliefs.add(Belief::new("language", "ZeroicAI is built entirely in Rust for type safety, performance, and zero-cost abstractions."));
    beliefs.add(Belief::new("license", "ZeroicAI is open source under the MIT and Apache-2.0 dual license."));
    beliefs.add(Belief::new("github", "ZeroicAI source code is available at https://github.com/ZeroicAI"));
    beliefs.add(Belief::new("website", "Learn more at https://zeroicai.xyz"));
    beliefs.add(Belief::new("telegram", "Join the ZeroicAI community on Telegram: https://t.me/ZeroicAI"));
    beliefs.add(Belief::new("twitter", "Follow ZeroicAI on X: https://x.com/ZeroicAI"));
    beliefs.add(Belief::new("owner", "ZeroicAI was created by Daniel, a full-stack developer based in the USA. He designed the framework architecture from the ground up, built all five core crates in Rust, and is leading the project toward a stable 1.0 release. You can find him on GitHub at github.com/pycogram and follow the project at github.com/ZeroicAI."));

    // Crate structure
    beliefs.add(Belief::new("crates", "ZeroicAI has 5 core crates: z-core, z-messaging, z-cognition, z-patterns, and z-runtime."));
    beliefs.add(Belief::new("core_crate", "z-core defines the Agent trait, AgentId, AgentContext, AgentState lifecycle, and error handling."));
    beliefs.add(Belief::new("messaging_crate", "z-messaging provides Router, Message, MessageBuilder, and FIPA performatives for agent communication."));
    beliefs.add(Belief::new("cognition_crate", "z-cognition provides BDI architecture (beliefs, desires, intentions), planning, reasoning, and utility functions."));
    beliefs.add(Belief::new("patterns_crate", "z-patterns provides 8 organizational patterns for structuring multi-agent systems."));
    beliefs.add(Belief::new("runtime_crate", "z-runtime provides scheduling, supervision, circuit breakers, metrics, and agent isolation."));

    // Agent trait
    beliefs.add(Belief::new("agent_trait", "Every agent implements the Agent trait with three async methods: initialize(), execute(), and shutdown()."));
    beliefs.add(Belief::new("agent_id", "Each agent has a unique UUID-based AgentId created with AgentId::new()."));
    beliefs.add(Belief::new("agent_state", "Agent lifecycle states: Created, Initialized, Running, Paused, Stopped. Transitions are validated."));

    // Messaging
    beliefs.add(Belief::new("messaging", "Agents communicate through a Router using typed Messages with FIPA performatives like Inform, Request, Propose, Accept, and Reject."));
    beliefs.add(Belief::new("performatives", "Supported FIPA performatives: Inform, Request, Query, Propose, Accept, Reject, Confirm, Disconfirm, Subscribe, CFP, Refuse."));
    beliefs.add(Belief::new("router", "The Router handles message delivery. Agents register to get a receiver channel, then messages are routed by AgentId."));

    // Cognition
    beliefs.add(Belief::new("bdi", "BDI stands for Belief-Desire-Intention. It's a cognitive architecture where agents maintain beliefs about the world, desires they want to achieve, and intentions they're pursuing."));
    beliefs.add(Belief::new("beliefs", "BeliefBase is a queryable knowledge store. Agents add, query, and remove beliefs as they learn about their environment."));
    beliefs.add(Belief::new("utility", "UtilityFunction maps states to numerical scores for strategy evaluation and decision making."));
    beliefs.add(Belief::new("planning", "The Planner supports state-action planning with preconditions and effects."));
    beliefs.add(Belief::new("reasoning", "The ReasoningEngine performs rule-based inference using if-then rules."));

    // Patterns
    beliefs.add(Belief::new("patterns", "ZeroicAI supports 8 organizational patterns: Hierarchy, Swarm, Coalition, Market, Blackboard, Federation, Holarchy, and Team."));
    beliefs.add(Belief::new("hierarchy", "Hierarchy pattern: command chains with Strategic, Tactical, and Operational levels. Tasks delegate down the chain."));
    beliefs.add(Belief::new("swarm", "Swarm pattern: decentralized coordination with flocking (separation, alignment, cohesion), foraging, and consensus voting."));
    beliefs.add(Belief::new("coalition", "Coalition pattern: temporary alliances where agents join forces with a shared strategy and combined value."));
    beliefs.add(Belief::new("market", "Market pattern: resource allocation via auctions. Supports English, Dutch, Vickrey, and sealed-bid auction types."));
    beliefs.add(Belief::new("federation", "Federation pattern: governance with weighted voting, policies, thresholds, and rules."));
    beliefs.add(Belief::new("team", "Team pattern: role-based coordination with Leader, Coordinator, and Executor roles and responsibilities."));
    beliefs.add(Belief::new("holarchy", "Holarchy pattern: nested autonomous units (holons) that are both wholes and parts of larger systems."));
    beliefs.add(Belief::new("blackboard", "Blackboard pattern: shared knowledge space where multiple agents read and write information."));

    // Runtime
    beliefs.add(Belief::new("supervisor", "Supervisor monitors agent health and applies restart policies: Never, Always, OnFailure, or ExponentialBackoff."));
    beliefs.add(Belief::new("circuit_breaker", "CircuitBreaker prevents cascading failures. States: Closed (normal), Open (blocking), HalfOpen (testing recovery)."));
    beliefs.add(Belief::new("scheduler", "Scheduler manages task queues with FairShare, Priority, RoundRobin, and FCFS policies."));
    beliefs.add(Belief::new("metrics", "MetricsRegistry collects Counter, Gauge, and Histogram metrics with label support and JSON export."));
    beliefs.add(Belief::new("backoff", "ExponentialBackoff provides retry logic with configurable initial delay, max delay, and multiplier."));
    beliefs.add(Belief::new("sandbox", "Sandbox provides agent isolation with CPU quota, memory limits, thread limits, and network isolation."));

    // Getting started
    beliefs.add(Belief::new("install", "Add z-core to your Cargo.toml dependencies. Use async-trait and tokio for async support."));
    beliefs.add(Belief::new("examples", "8 working examples are available at https://github.com/ZeroicAI/z-examples covering all 5 crates."));
    beliefs.add(Belief::new("docs", "Documentation is available at https://zeroicai.xyz/docs"));

    // Solana integration
    beliefs.add(Belief::new("solana", "ZeroicAI has native Solana integration. Agents can transact, coordinate, and settle on-chain using the Solana network — sub-second finality, low fees."));
    beliefs.add(Belief::new("solana_usecase", "Use ZeroicAI on Solana for: autonomous DeFi agents, on-chain coordination, decentralized AI marketplaces, and trustless agent economies."));
    beliefs.add(Belief::new("defi_agents", "ZeroicAI agents can monitor liquidity pools, execute swaps, manage risk, and rebalance portfolios autonomously on Solana."));

    // Debate agents
    beliefs.add(Belief::new("debate_agents", "ZeroicAI runs a debate agent system on X: multiple agents with different personas (ZERO, AXIOM, NEXUS, CIPHER, VECTOR) debate trending topics in AI, crypto, and tech."));

    // Philosophy
    beliefs.add(Belief::new("why_rust", "Rust gives ZeroicAI type safety, zero-cost abstractions, fearless concurrency, and no garbage collector pauses."));
    beliefs.add(Belief::new("design", "ZeroicAI uses composition over inheritance, async-first design, zero-cost patterns, and fail-graceful architecture."));
    beliefs.add(Belief::new("fipa", "FIPA is the Foundation for Intelligent Physical Agents, an IEEE standard for agent communication that ZeroicAI implements."));
    beliefs.add(Belief::new("modular", "Use only what you need. A simple agent needs only z-core. Complex systems compose all five crates."));

    beliefs
}
