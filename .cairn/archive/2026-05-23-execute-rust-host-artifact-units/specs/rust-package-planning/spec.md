# Rust package planning host-artifact execution delta

## ADDED Requirements

### Requirement: Rust host-artifact topology execution

r[rust_package_planning.unit_execution.host_artifacts] Mantle MUST execute a bounded Rust host-artifact topology from explicit unit derivation graph evidence without invoking Cargo as the build orchestrator.

#### Scenario: Host artifacts execute before target consumers

r[rust_package_planning.unit_execution.host_artifacts.executes]

- GIVEN `unit_derivation_graph` is ready and contains supported host `proc-macro` or `custom-build` units plus supported target `lib`/`bin` consumers
- WHEN host-artifact topology execution is requested
- THEN Mantle MUST execute supported host units before target consumers using explicit derivation args/env.
- AND Mantle MUST NOT invoke Cargo as the build orchestrator for host or target execution.

#### Scenario: Host artifacts bind into target execution material

r[rust_package_planning.unit_execution.host_artifacts.binds]

- GIVEN a target unit consumes a host artifact represented in `consumed_host_artifacts` and matching dependency surfaces
- WHEN the producing host unit succeeds
- THEN Mantle MUST bind the host-produced artifact path into target consumed-host artifacts, derivation inputs, and matching `--extern` dependency surfaces before invoking target `rustc`.
- AND the target receipt MUST bind the host artifact digest for the produced artifact it consumed.

### Requirement: Rust host-artifact execution blockers

r[rust_package_planning.unit_execution.host_artifacts.blockers] Mantle MUST fail closed with deterministic blockers for unsupported or incomplete host-artifact execution before invoking an affected target `rustc`.

#### Scenario: Missing or unsupported host material blocks target execution

r[rust_package_planning.unit_execution.host_artifacts.blockers.missing_material]

- GIVEN a target unit consumes a host artifact
- WHEN the host producer is absent, unsupported, fails, emits no matching artifact, or the rebound host artifact is missing/unreadable
- THEN Mantle MUST emit a deterministic host-artifact topology blocker.
- AND Mantle MUST NOT search ambient Cargo target directories or caches to repair host material.

### Requirement: Rust host-artifact execution receipts

r[rust_package_planning.unit_execution_receipts.host_artifacts] Mantle MUST emit host-artifact topology execution receipts that preserve ordered host and target per-unit execution evidence.

#### Scenario: Receipt preserves ordered host and target evidence

r[rust_package_planning.unit_execution_receipts.host_artifacts.ordered]

- GIVEN host-artifact topology execution is requested
- WHEN Mantle reports the result
- THEN the receipt MUST include ordered host and target unit execution receipts, output artifact BLAKE3 digests, host artifact BLAKE3 digests for consumers, a bounded host-artifact claim, an optional blocker, and a stable receipt hash.

### Requirement: Rust host-artifact CLI receipts

r[rust_package_planning.unit_execution_receipts.host_artifacts.cli] Mantle MUST expose host-artifact topology execution as reviewable `rust-plan` CLI JSON evidence.

#### Scenario: CLI emits captured plan plus host-artifact execution receipt

r[rust_package_planning.unit_execution_receipts.host_artifacts.cli.json]

- GIVEN `mantle rust-plan` captures a ready unit graph with supported host artifacts and target consumers
- WHEN the host-artifact topology CLI flag is requested with an explicit execution output root
- THEN Mantle MUST emit JSON containing the captured Rust plan receipt and the host-artifact topology execution receipt.
