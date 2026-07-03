# Rust Package Planning Specification

## Purpose

Defines requirements for making native rust-plan validation deterministic under repeated serial and parallel execution.

## Requirements

### Requirement: Native topology validation is stable

r[rust_package_planning.native_topology_validation_stability] Native Rust topology validation SHOULD provide focused commands that pass deterministically under documented host tooling and do not rely on hidden fixture ordering.

#### Scenario: Focused serial rail passes

GIVEN the documented Mantle Rust build environment is available
WHEN the focused native rust-plan serial validation command runs
THEN it MUST pass or report a deterministic first-party blocker
AND the evidence MUST include the exact command output summary.

#### Scenario: Repeated validation has the same result

GIVEN no source files change between runs
WHEN the focused validation command is repeated
THEN it SHOULD produce the same pass/blocker status
AND it MUST NOT depend on leftover fixture state from a previous run.

### Requirement: Native topology fixtures are race-free

r[rust_package_planning.native_topology_race_free_fixtures] Native rust-plan tests MUST isolate or explicitly lock shared fixture state such as rustc wrappers, cargo shims, OUT_DIRs, execution roots, compiler-policy files, and ambient environment probes.

#### Scenario: Parallel fixtures do not share output roots

GIVEN two native topology tests run concurrently
WHEN they create compiler wrappers, cargo shims, or execution output directories
THEN each test MUST use a unique root or a documented lock
AND one test MUST NOT observe another test's generated artifacts.

#### Scenario: Ambient environment tests are subprocessed

GIVEN a negative test needs conflicting environment variables
WHEN it proves planner behavior under those variables
THEN it MUST spawn a child process or use an equivalent isolation boundary
AND it MUST NOT mutate process-global environment in a way that races with other tests.

### Requirement: Native topology parallel evidence is explicit

r[rust_package_planning.native_topology_parallel_evidence] Mantle validation evidence MUST distinguish serial-only, parallel-safe, stress-tested, and remaining-blocked native rust-plan rails.

#### Scenario: Parallel rail is claimed only after proof

GIVEN a summary claims native rust-plan focused tests are parallel-safe
WHEN that summary is written
THEN it MUST cite same-run parallel or repeated-run evidence
AND it MUST NOT generalize from a serial-only run.

#### Scenario: Remaining serial-only test is documented

GIVEN a native rust-plan test still requires serialization
WHEN validation docs are updated
THEN the docs MUST name the shared resource or race risk
AND they MUST identify a next action for removing the serial requirement.
