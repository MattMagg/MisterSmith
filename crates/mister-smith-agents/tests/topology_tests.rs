use mister_smith_agents::{TopologyCompiler, TopologySignals};
use mister_smith_core::{CoordinationPolicy, HealthState, TaskId, TopologyKind};
use serde_json::json;

fn independent_branches_plan() -> serde_json::Value {
    json!({
        "goal": "independent-branches",
        "steps": [
            {
                "id": "root",
                "step": 1,
                "action": "root-step",
                "description": "Root step"
            },
            {
                "id": "left",
                "step": 2,
                "action": "left-branch",
                "description": "Left branch",
                "depends_on": ["root"],
                "branch": "left"
            },
            {
                "id": "right",
                "step": 3,
                "action": "right-branch",
                "description": "Right branch",
                "depends_on": ["root"],
                "branch": "right"
            }
        ]
    })
}

fn strict_chain_plan() -> serde_json::Value {
    json!({
        "goal": "strict-chain",
        "steps": [
            {
                "id": "step-1",
                "step": 1,
                "action": "first",
                "description": "First"
            },
            {
                "id": "step-2",
                "step": 2,
                "action": "second",
                "description": "Second",
                "depends_on": ["step-1"]
            },
            {
                "id": "step-3",
                "step": 3,
                "action": "third",
                "description": "Third",
                "depends_on": ["step-2"]
            }
        ]
    })
}

fn join_plan() -> serde_json::Value {
    json!({
        "goal": "join-workflow",
        "steps": [
            {
                "id": "root",
                "step": 1,
                "action": "root",
                "description": "Root"
            },
            {
                "id": "left",
                "step": 2,
                "action": "left",
                "description": "Left",
                "depends_on": ["root"],
                "branch": "left"
            },
            {
                "id": "right",
                "step": 3,
                "action": "right",
                "description": "Right",
                "depends_on": ["root"],
                "branch": "right"
            },
            {
                "id": "join",
                "step": 4,
                "action": "join",
                "description": "Join",
                "depends_on": ["left", "right"]
            }
        ]
    })
}

fn hierarchical_hint_plan() -> serde_json::Value {
    json!({
        "goal": "hierarchical-review",
        "topology_hint": "hierarchical",
        "steps": [
            {
                "id": "coordinate",
                "step": 1,
                "action": "coordinate",
                "description": "Coordinate subtree execution"
            },
            {
                "id": "review-a",
                "step": 2,
                "action": "review-a",
                "description": "Review branch A",
                "depends_on": ["coordinate"],
                "branch": "subtree-a"
            },
            {
                "id": "review-b",
                "step": 3,
                "action": "review-b",
                "description": "Review branch B",
                "depends_on": ["coordinate"],
                "branch": "subtree-b"
            }
        ]
    })
}

#[test]
fn topology_compiler_selects_parallel_for_independent_branches() {
    let compiler = TopologyCompiler::default();
    let graph = compiler
        .compile(
            TaskId::new(),
            &independent_branches_plan(),
            &TopologySignals::default(),
        )
        .expect("independent branches should compile");

    assert_eq!(graph.topology_plan.topology_kind, TopologyKind::Parallel);
    assert_eq!(
        graph.topology_plan.coordination_policy,
        CoordinationPolicy::Barrier
    );
    assert!(!graph.topology_plan.rationale.dependency_shape.is_empty());
    assert!(!graph.topology_plan.rationale.operational_signals.is_empty());
    assert_eq!(
        graph.topology_plan.fallback_topology,
        Some(TopologyKind::Sequential)
    );
}

#[test]
fn topology_compiler_selects_sequential_for_strict_chain() {
    let compiler = TopologyCompiler::default();
    let graph = compiler
        .compile(
            TaskId::new(),
            &strict_chain_plan(),
            &TopologySignals::default(),
        )
        .expect("strict chain should compile");

    assert_eq!(graph.topology_plan.topology_kind, TopologyKind::Sequential);
    assert_eq!(
        graph.topology_plan.coordination_policy,
        CoordinationPolicy::StrictSequence
    );
    assert_eq!(graph.topology_plan.fallback_topology, None);
}

#[test]
fn topology_compiler_selects_pipeline_for_streaming_chain() {
    let compiler = TopologyCompiler::default();
    let signals = TopologySignals {
        prefer_streaming: true,
        ..TopologySignals::default()
    };
    let graph = compiler
        .compile(TaskId::new(), &strict_chain_plan(), &signals)
        .expect("streaming chain should compile");

    assert_eq!(graph.topology_plan.topology_kind, TopologyKind::Pipeline);
    assert_eq!(
        graph.topology_plan.coordination_policy,
        CoordinationPolicy::Streaming
    );
    assert_eq!(
        graph.topology_plan.fallback_topology,
        Some(TopologyKind::Sequential)
    );
}

#[test]
fn topology_compiler_selects_hybrid_for_join_graph() {
    let compiler = TopologyCompiler::default();
    let graph = compiler
        .compile(TaskId::new(), &join_plan(), &TopologySignals::default())
        .expect("join graph should compile");

    assert_eq!(graph.topology_plan.topology_kind, TopologyKind::Hybrid);
    assert_eq!(
        graph.topology_plan.coordination_policy,
        CoordinationPolicy::Mixed
    );
    assert_eq!(
        graph.topology_plan.fallback_topology,
        Some(TopologyKind::Sequential)
    );
}

#[test]
fn topology_compiler_honors_compatible_hierarchical_hint() {
    let compiler = TopologyCompiler::default();
    let signals = TopologySignals {
        health_state: Some(HealthState::Healthy),
        ..TopologySignals::default()
    };
    let graph = compiler
        .compile(TaskId::new(), &hierarchical_hint_plan(), &signals)
        .expect("hierarchical hint should compile");

    assert_eq!(
        graph.topology_plan.topology_kind,
        TopologyKind::Hierarchical
    );
    assert_eq!(
        graph.topology_plan.coordination_policy,
        CoordinationPolicy::HierarchicalReduce
    );
    assert!(graph
        .topology_plan
        .rationale
        .selected_for
        .contains("hierarchical"));
}
