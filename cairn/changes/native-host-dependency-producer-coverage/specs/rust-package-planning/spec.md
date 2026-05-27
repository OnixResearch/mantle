# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Native host dependency producer coverage

r[rust_package_planning.native_host_dependency_producer_coverage] Mantle MUST provide bounded producer coverage for selected host-unit dependency artifacts before claiming unified topology execution readiness.

#### Scenario: Proc-macro host dependency producer executes before host consumer

GIVEN a selected native host unit consumes a dependency artifact whose package has a supported native proc-macro host producer
AND the dependency package has ready source, package, host-unit, and derivation graph facts
WHEN Mantle executes unified topology evidence
THEN Mantle MUST execute the proc-macro host producer before the consuming host unit.
AND Mantle MUST bind the produced proc-macro host artifact path into the consuming host unit's dependency artifact surface before invoking `rustc`.

#### Scenario: Target library host dependency producer remains supported

GIVEN a selected native host unit consumes a dependency artifact whose package has a supported native target `lib` producer
WHEN Mantle computes unified topology ordering
THEN Mantle MUST execute the target `lib` producer and its target dependencies before the consuming host unit.
AND Mantle MUST bind the produced target library artifact path into the consuming host unit's dependency artifact surface before invoking `rustc`.

#### Scenario: Missing host dependency producer fails closed

GIVEN a selected native host unit consumes a dependency artifact whose package has neither a supported target `lib` producer nor a supported proc-macro host producer
WHEN Mantle computes unified topology ordering
THEN Mantle MUST return a deterministic `missing-host-dependency-producer` blocker naming the dependency package.
AND Mantle MUST NOT invoke the host consumer using ambient Cargo state, registry caches, target directories, or network locations.

#### Scenario: Self-probe moves past rustversion host dependency producer coverage

GIVEN the current self probe reports `missing-host-dependency-producer` for `registry+https://github.com/rust-lang/crates.io-index#rustversion@1.0.22`
WHEN native host dependency producer coverage is implemented
THEN focused verification MUST show that `rustversion@1.0.22` no longer fails solely because its proc-macro host producer is ignored.
AND remaining blockers, if any, MUST be recorded with deterministic classes and baseline/current evidence.
