use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use crunch_build::ContentLocalityClass;
use crunch_build::HardEligibilityFacts;
use crunch_build::KnownGraphPressure;
use crunch_build::ReadyGoalFacts;
use crunch_build::ResourceFitClass;
use crunch_build::SchedulingPolicy;
use crunch_build::TransferCostClass;
use serde::Serialize;

const BENCHMARK_ITERATIONS: u32 = 20_000;
const STRESS_BENCHMARK_ITERATIONS: u32 = 200;
const STRESS_LOW_PRIORITY_GOAL_COUNT: u32 = 1_023;
const STRESS_READY_GOAL_COUNT: u32 = 1_024;
const BENCHMARK_EPOCH: u32 = 8;
const LONG_PATH_NODES: u32 = 8;
const SHORT_PATH_NODES: u32 = 1;
const SHARED_BLOCKED_ROOTS: u32 = 3;
const SINGLE_BLOCKED_ROOT: u32 = 1;

#[derive(Clone)]
struct SchedulerFixture {
    name: &'static str,
    ready: Vec<ReadyGoalFacts>,
    pressures: BTreeMap<String, KnownGraphPressure>,
    expected_priority_first: &'static str,
    comparison_basis: &'static str,
    iterations: u32,
}

#[derive(Debug, Serialize)]
struct FixtureComparison {
    name: String,
    comparison_basis: String,
    iterations: u32,
    ready_goal_count: u32,
    fifo_first: String,
    priority_first: String,
    fifo_first_blocked_roots: u32,
    priority_first_blocked_roots: u32,
    fifo_runtime_ns_total: u128,
    priority_runtime_ns_total: u128,
}

#[derive(Debug, Serialize)]
struct SchedulerBenchmarkReport {
    schema: &'static str,
    default_iterations: u32,
    measurement: &'static str,
    policy_id: String,
    policy_digest_blake3: String,
    claim_scope: &'static str,
    non_claims: Vec<&'static str>,
    fixtures: Vec<FixtureComparison>,
}

fn pressure(path_nodes: u32, blocked_roots: u32) -> KnownGraphPressure {
    assert!(path_nodes > 0);
    assert!(blocked_roots > 0);
    KnownGraphPressure {
        known_critical_path_nodes: path_nodes,
        known_critical_path_work_units: path_nodes,
        blocked_root_count: blocked_roots,
        blocked_root_count_saturated: false,
    }
}

fn ordinary(key: &str) -> ReadyGoalFacts {
    assert!(!key.is_empty());
    assert!(!key.chars().any(char::is_control));
    ReadyGoalFacts::ordinary(key.to_string(), 0)
}

fn diamond_fixture() -> SchedulerFixture {
    let fixture = SchedulerFixture {
        name: "diamond-deep-branch-vs-shallow",
        ready: vec![ordinary("diamond-shallow-leaf"), ordinary("diamond-deep-leaf")],
        pressures: BTreeMap::from([
            ("diamond-shallow-leaf".to_string(), pressure(SHORT_PATH_NODES, SINGLE_BLOCKED_ROOT)),
            ("diamond-deep-leaf".to_string(), pressure(LONG_PATH_NODES, SINGLE_BLOCKED_ROOT)),
        ]),
        expected_priority_first: "diamond-deep-leaf",
        comparison_basis: "diamond-known-critical-path",
        iterations: BENCHMARK_ITERATIONS,
    };
    assert_eq!(fixture.ready.len(), fixture.pressures.len());
    assert!(!fixture.ready.is_empty());
    fixture
}

fn stress_fixture() -> SchedulerFixture {
    let capacity = usize::try_from(STRESS_READY_GOAL_COUNT).expect("stress count fits usize");
    let mut ready = Vec::with_capacity(capacity);
    let mut pressures = BTreeMap::new();
    for index in 0..STRESS_LOW_PRIORITY_GOAL_COUNT {
        let key = format!("stress-ordinary-{index}");
        ready.push(ordinary(&key));
        pressures.insert(key, pressure(SHORT_PATH_NODES, SINGLE_BLOCKED_ROOT));
    }
    let preferred_key = "stress-priority";
    ready.push(ordinary(preferred_key));
    pressures.insert(preferred_key.to_string(), pressure(LONG_PATH_NODES, SHARED_BLOCKED_ROOTS));
    assert_eq!(u32::try_from(ready.len()).unwrap(), STRESS_READY_GOAL_COUNT);
    assert_eq!(STRESS_LOW_PRIORITY_GOAL_COUNT.checked_add(1), Some(STRESS_READY_GOAL_COUNT));
    SchedulerFixture {
        name: "bounded-ready-set-stress",
        ready,
        pressures,
        expected_priority_first: preferred_key,
        comparison_basis: "bounded-ready-set-stress",
        iterations: STRESS_BENCHMARK_ITERATIONS,
    }
}

fn benchmark_fixtures() -> Vec<SchedulerFixture> {
    let chain = SchedulerFixture {
        name: "chain-vs-short-root",
        ready: vec![ordinary("short-root"), ordinary("long-chain-leaf")],
        pressures: BTreeMap::from([
            ("short-root".to_string(), pressure(SHORT_PATH_NODES, SINGLE_BLOCKED_ROOT)),
            ("long-chain-leaf".to_string(), pressure(LONG_PATH_NODES, SINGLE_BLOCKED_ROOT)),
        ]),
        expected_priority_first: "long-chain-leaf",
        comparison_basis: "known-critical-path",
        iterations: BENCHMARK_ITERATIONS,
    };
    let shared = SchedulerFixture {
        name: "shared-dependency-vs-independent",
        ready: vec![ordinary("independent-leaf"), ordinary("diamond-shared-leaf")],
        pressures: BTreeMap::from([
            ("independent-leaf".to_string(), pressure(SHORT_PATH_NODES, SINGLE_BLOCKED_ROOT)),
            ("diamond-shared-leaf".to_string(), pressure(LONG_PATH_NODES, SHARED_BLOCKED_ROOTS)),
        ]),
        expected_priority_first: "diamond-shared-leaf",
        comparison_basis: "shared-blocked-root-pressure",
        iterations: BENCHMARK_ITERATIONS,
    };
    let mut local = ordinary("local-compatible");
    local.preference = crunch_build::normalize_eligible_preference(
        HardEligibilityFacts::ELIGIBLE,
        ResourceFitClass::Exact,
        ContentLocalityClass::FullyPresent,
        TransferCostClass::None,
    )
    .expect("eligible benchmark route");
    let locality = SchedulerFixture {
        name: "verified-locality-vs-unknown",
        ready: vec![ordinary("remote-unknown"), local],
        pressures: BTreeMap::from([
            ("remote-unknown".to_string(), pressure(SHORT_PATH_NODES, SINGLE_BLOCKED_ROOT)),
            ("local-compatible".to_string(), pressure(SHORT_PATH_NODES, SINGLE_BLOCKED_ROOT)),
        ]),
        expected_priority_first: "local-compatible",
        comparison_basis: "resource-locality-transfer",
        iterations: BENCHMARK_ITERATIONS,
    };
    let fairness = SchedulerFixture {
        name: "aged-goal-vs-recurring-pressure",
        ready: vec![
            ReadyGoalFacts::ordinary("new-high-pressure".to_string(), BENCHMARK_EPOCH),
            ordinary("continuously-ready"),
        ],
        pressures: BTreeMap::from([
            ("new-high-pressure".to_string(), pressure(LONG_PATH_NODES, SHARED_BLOCKED_ROOTS)),
            ("continuously-ready".to_string(), pressure(SHORT_PATH_NODES, SINGLE_BLOCKED_ROOT)),
        ]),
        expected_priority_first: "continuously-ready",
        comparison_basis: "deterministic-starvation-class",
        iterations: BENCHMARK_ITERATIONS,
    };
    let diamond = diamond_fixture();
    let stress = stress_fixture();
    let fixtures = vec![chain, diamond, shared, locality, fairness, stress];
    assert!(!fixtures.is_empty());
    assert!(fixtures.iter().all(|fixture| !fixture.ready.is_empty()));
    fixtures
}

fn fifo_first(fixture: &SchedulerFixture) -> String {
    let first = fixture.ready.first().expect("benchmark fixture has a ready goal").goal_key.clone();
    assert!(!fixture.ready.is_empty());
    assert!(!first.is_empty());
    first
}

fn priority_order(
    policy: &SchedulingPolicy,
    fixture: &SchedulerFixture,
) -> Result<Vec<String>, crunch_build::SchedulingError> {
    let ranked = crunch_build::rank_ready_goals(policy, BENCHMARK_EPOCH, &fixture.ready, &fixture.pressures)?;
    let order: Vec<String> = ranked.into_iter().map(|goal| goal.goal_key).collect();
    assert_eq!(order.len(), fixture.ready.len());
    assert!(!order.is_empty());
    Ok(order)
}

fn measure_fifo_ns(fixture: &SchedulerFixture) -> u128 {
    let started = Instant::now();
    for _ in 0..fixture.iterations {
        black_box(fifo_first(black_box(fixture)));
    }
    let elapsed_ns = started.elapsed().as_nanos();
    assert!(elapsed_ns > 0);
    assert!(fixture.iterations > 0);
    elapsed_ns
}

fn measure_priority_ns(policy: &SchedulingPolicy, fixture: &SchedulerFixture) -> u128 {
    let started = Instant::now();
    for _ in 0..fixture.iterations {
        black_box(priority_order(black_box(policy), black_box(fixture)).expect("valid benchmark facts"));
    }
    let elapsed_ns = started.elapsed().as_nanos();
    assert!(elapsed_ns > 0);
    assert!(fixture.iterations > 0);
    elapsed_ns
}

fn compare_fixture(policy: &SchedulingPolicy, fixture: &SchedulerFixture) -> FixtureComparison {
    let fifo_first = fifo_first(fixture);
    let priority = priority_order(policy, fixture).expect("valid benchmark fixture");
    assert_eq!(priority[0], fixture.expected_priority_first);
    assert_ne!(fifo_first, priority[0]);
    FixtureComparison {
        name: fixture.name.to_string(),
        comparison_basis: fixture.comparison_basis.to_string(),
        iterations: fixture.iterations,
        ready_goal_count: u32::try_from(fixture.ready.len()).expect("bounded ready count"),
        fifo_first_blocked_roots: fixture.pressures[&fifo_first].blocked_root_count,
        priority_first_blocked_roots: fixture.pressures[&priority[0]].blocked_root_count,
        fifo_first,
        priority_first: priority[0].clone(),
        fifo_runtime_ns_total: measure_fifo_ns(fixture),
        priority_runtime_ns_total: measure_priority_ns(policy, fixture),
    }
}

fn build_report() -> SchedulerBenchmarkReport {
    let policy = SchedulingPolicy::default();
    let fixtures = benchmark_fixtures().iter().map(|fixture| compare_fixture(&policy, fixture)).collect();
    SchedulerBenchmarkReport {
        schema: "mantle-scheduler-benchmark-v1",
        default_iterations: BENCHMARK_ITERATIONS,
        measurement: "debug-build-ready-selection-monotonic-total-nanoseconds",
        policy_id: policy.policy_id.clone(),
        policy_digest_blake3: policy.digest_blake3().expect("valid default policy"),
        claim_scope: "comparative-fixture-ordering-and-runtime",
        non_claims: vec![
            "global-optimality",
            "production-throughput",
            "future-graph-knowledge",
            "execution-success",
            "graph-snapshot-or-pressure-recomputation-overhead",
        ],
        fixtures,
    }
}

fn main() {
    let report = build_report();
    assert!(!report.fixtures.is_empty());
    assert!(report.fixtures.iter().all(|fixture| fixture.priority_runtime_ns_total > 0));
    println!("{}", serde_json::to_string_pretty(&report).expect("serialize benchmark report"));
}
