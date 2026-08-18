### Requirement: Rust topology output reuse

r[rust_package_planning.topology_output_reuse] Mantle MUST explain rebuild versus reuse for supported Rust topology unit outputs using explicit per-unit execution receipt material.

#### Scenario: Prior topology outputs are discovered from the explicit output root

r[rust_package_planning.topology_output_reuse.lookup]

- GIVEN `rust-plan --execute-topology` is requested with an execution output root
- AND the output root contains prior per-unit topology execution receipt material
- WHEN Mantle evaluates a supported unit for execution
- THEN Mantle MUST load only receipt material from that explicit output root as candidate reuse evidence.
- AND Mantle MUST NOT search Cargo target directories, registry caches, git checkouts, or undeclared cache locations to find reusable outputs.

#### Scenario: Matching prior outputs are reused

r[rust_package_planning.topology_output_reuse.match]

- GIVEN a prior per-unit topology execution receipt names declared output artifacts
- AND those declared output artifacts are present and readable under the current execution output root
- WHEN the current unit identity, source closure digest, dependency artifact digests, host artifact digests, toolchain identity, `rustc` argument digest, declared output paths, and output artifact BLAKE3 digests match the prior receipt material
- THEN Mantle MUST report successful unit execution with a reuse rebuild reason instead of invoking `rustc` again for that unit.
- AND the reused receipt MUST bind the current output artifact BLAKE3 digests.

#### Scenario: Stale cached output blocks before rustc

r[rust_package_planning.topology_output_reuse.blockers]

- GIVEN prior topology output receipt material exists for a unit
- WHEN the prior receipt is malformed, unreadable, names a missing declared output, names an unreadable output, has digest-mismatched output material, or no longer matches the current explicit unit inputs
- THEN Mantle MUST emit a deterministic stale-cache blocker before invoking `rustc` for that unit.
- AND Mantle MUST NOT silently fall back to Cargo, ambient caches, or an unreviewed rebuild for that stale cached unit.

#### Scenario: Rebuild and reuse decisions are receipt-bound

r[rust_package_planning.topology_output_reuse.receipts]

- GIVEN topology execution succeeds or fails closed for a supported graph boundary
- WHEN Mantle emits the combined JSON receipt
- THEN the receipt MUST preserve per-unit rebuild versus reuse reasons, ordered unit execution evidence, output artifact BLAKE3 digests, stale-cache blockers when present, and stable receipt hashes.
- AND the receipt MUST keep the bounded Cargo-free topology claim explicit.

#### Scenario: Topology output reuse behavior is covered by focused fixtures

r[rust_package_planning.topology_output_reuse.tests]

- GIVEN Mantle includes focused `rust_plan_cli` fixtures for topology output reuse
- WHEN tests exercise repeated topology execution and stale cached output material
- THEN the positive fixtures MUST prove matching prior outputs are reused without invoking Cargo as the build orchestrator.
- AND the negative fixtures MUST prove stale prior evidence yields a deterministic pre-`rustc` blocker for the affected unit.

#### Scenario: Topology output reuse change closes with lifecycle evidence

r[rust_package_planning.topology_output_reuse.verify]

- GIVEN the topology output reuse implementation tasks are complete
- WHEN Mantle accepts the change
- THEN focused Rust verification, Cairn validation, and proposal/design/tasks gates MUST pass before sync, archive, commit, and push.
