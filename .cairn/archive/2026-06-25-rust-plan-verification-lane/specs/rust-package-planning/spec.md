## ADDED Requirements

### Requirement: Rust-plan remains an explicit verification lane [r[rust_package_planning.verification_lane_boundary]]

Mantle MUST keep native `rust-plan` execution as an explicit bounded verification lane until separate promotion evidence justifies using it as a default project-build path, and MUST keep rust-plan claims machine-readable and narrower than the evidence.

#### Scenario: Default project builds do not silently use rust-plan [r[rust_package_planning.verification_lane_boundary.scenario.default-build]]

- GIVEN a Mantle project declares a Rust package for the supported offline Cargo build lane
- WHEN the operator runs `mantle build .#name` without an explicit native-planner opt-in
- THEN Mantle MUST use the declared project build workflow rather than silently invoking native rust-plan topology execution
- AND the resulting report MUST NOT claim Cargo-free execution.

#### Scenario: Explicit rust-plan emits bounded evidence [r[rust_package_planning.verification_lane_boundary.scenario.explicit]]

- GIVEN the operator explicitly invokes `mantle rust-plan` with a supported execution flag
- WHEN native planning or topology execution succeeds
- THEN Mantle MUST emit deterministic receipt evidence that names the supported graph, source facts, toolchain identity, output digests, and claim class
- AND the claim MUST remain bounded to the explicit units or topology that were executed.

#### Scenario: Unsupported native planner surfaces fail closed [r[rust_package_planning.verification_lane_boundary.scenario.unsupported]]

- GIVEN a Rust workspace requires Cargo behavior outside Mantle's supported native-planner surface
- WHEN rust-plan planning or execution evaluates that workspace
- THEN Mantle MUST emit deterministic blockers naming the unsupported surface
- AND it MUST NOT invoke Cargo as hidden build orchestration while claiming Cargo-free success.

#### Scenario: Promotion requires current evidence [r[rust_package_planning.verification_lane_boundary.scenario.promotion]]

- GIVEN a future change proposes to make native rust-plan execution a default or broader project-build path
- WHEN that change is reviewed
- THEN it MUST cite current compatibility-rail evidence, positive and negative fixture coverage, bounded report wording, and validation receipts
- AND it MUST preserve non-claims for full Cargo compatibility, compiler correctness, release reproducibility, and bootstrap correctness unless separate evidence proves them.
