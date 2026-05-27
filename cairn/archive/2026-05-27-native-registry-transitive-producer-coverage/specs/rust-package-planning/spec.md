# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Native registry transitive producer coverage

r[rust_package_planning.native_registry_transitive_producer_coverage] Mantle MUST provide executable native producer coverage for supported transitive registry dependency artifacts before claiming unit-derivation graph or topology readiness.

#### Scenario: Supported transitive registry dependency gets a producer unit

GIVEN native package-target planning has ready facts for a registry-backed dependency package with ready native source facts and a supported `lib` target
AND a supported native target unit consumes a dependency artifact for that registry package
WHEN Mantle computes native unit graph and unit-derivation graph evidence
THEN Mantle MUST include an eligible producer `lib` unit for that registry package or otherwise keep graph readiness false with a deterministic blocker.
AND the producer unit MUST bind the package identity, source digest, rustc argument digest, declared output, and dependency artifacts using explicit native facts.

#### Scenario: Missing producer coverage fails before topology execution

GIVEN a supported consumer unit has a dependency artifact whose package lacks native package facts, ready source facts, a supported `lib` target, or a selected build-mode producer unit
WHEN Mantle computes native unit graph or unit-derivation graph evidence
THEN Mantle MUST emit a deterministic planning blocker naming the consumer package, dependency package, and missing producer reason.
AND Mantle MUST NOT report `unit_derivation_graph.ready=true`, native unified topology readiness, or topology execution readiness for that graph.

#### Scenario: Transitive producer execution remains source-closure bounded

GIVEN `rust-plan --execute-topology` executes a graph containing a transitive registry-backed producer
WHEN Mantle executes the producer and downstream consumers
THEN Mantle MUST execute the producer before affected consumers using only explicit derivation args, source facts, BLAKE3 source digests, dependency artifacts, host artifacts, and declared outputs.
AND Mantle MUST NOT use Cargo as the build orchestrator or read undeclared registry caches, git checkouts, target directories, or network locations to materialize the producer.

#### Scenario: Self-probe blocker moves past itertools producer coverage

GIVEN the current pushed-head self probe reports `missing-dependency-producer` for `registry+https://github.com/rust-lang/crates.io-index#itertools@0.10.5`
WHEN native registry transitive producer coverage is implemented
THEN focused verification MUST show that `itertools@0.10.5` no longer fails solely because no producer unit exists.
AND remaining blockers, if any, MUST be recorded with deterministic classes and baseline/current evidence.
