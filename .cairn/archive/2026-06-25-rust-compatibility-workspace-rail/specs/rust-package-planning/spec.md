## ADDED Requirements

### Requirement: Representative Rust compatibility workspace rail [r[rust_package_planning.compatibility_workspace_rail]]

Mantle MUST maintain a representative Rust workspace rail before making broad practical Rust project support claims, and MUST distinguish sandboxed offline Cargo results from native rust-plan results for that rail.

#### Scenario: Rail fixture covers practical Rust surfaces [r[rust_package_planning.compatibility_workspace_rail.scenario.fixture]]

- GIVEN Mantle reports representative Rust project-build support
- WHEN the compatibility rail fixture is inspected
- THEN the fixture MUST include a runnable binary, local library dependency, vendored registry dependency, proc-macro host artifact, build-script metadata surface, and explicit source-closure facts
- AND the fixture MUST run without external network or ambient Cargo cache access in fast validation.

#### Scenario: Offline Cargo lane proves practical build for the fixture [r[rust_package_planning.compatibility_workspace_rail.scenario.offline-cargo]]

- GIVEN the representative fixture has complete declared source material
- WHEN the offline Cargo project build lane validates it
- THEN Mantle MUST build the fixture in the sandbox, run the declared binary smoke, and emit report evidence binding source closure, toolchain, output, and artifact attestation
- AND the evidence MUST identify the result as Cargo-orchestrated inside Mantle rather than Cargo-free native execution.

#### Scenario: Rust-plan lane reports bounded success or blockers [r[rust_package_planning.compatibility_workspace_rail.scenario.rust-plan]]

- GIVEN the representative fixture is evaluated through explicit `mantle rust-plan` topology execution
- WHEN native planning or execution reaches an unsupported surface
- THEN Mantle MUST emit a deterministic blocker naming that surface and preserving partial evidence when available
- AND it MUST NOT fall back to Cargo while claiming Cargo-free success.

#### Scenario: Compatibility claims are evidence-scoped [r[rust_package_planning.compatibility_workspace_rail.scenario.claims]]

- GIVEN a task, README, release note, or status reply cites the representative rail
- WHEN it states what Mantle can build
- THEN the claim MUST identify which lane passed, which fixture was inspected, and which evidence file or command output proves it
- AND it MUST NOT generalize that result to full Cargo compatibility, compiler correctness, release reproducibility, or bootstrap correctness.
