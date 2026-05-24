### Requirement: Rust topology output reuse

r[rust_package_planning.unit_execution.topology.output_reuse] Mantle MUST explain rebuild versus reuse for supported Rust topology unit outputs using explicit receipt material.

#### Scenario: Repeated topology execution reuses matching outputs

r[rust_package_planning.unit_execution.topology.output_reuse.repeated]

- GIVEN `rust-plan --execute-topology` has already produced declared output artifacts and per-unit execution receipts under an execution output root
- WHEN the same explicit Rust topology is executed again with matching source digests, toolchain identity, rustc args digest, dependency artifact digests, host artifact digests, declared outputs, and output artifact BLAKE3 digests
- THEN Mantle MUST report successful unit execution with a reuse rebuild reason instead of invoking `rustc` again for that unit.
- AND the reused receipt MUST bind the current output artifact BLAKE3 digests.

### Requirement: Rust topology output reuse blockers

r[rust_package_planning.unit_execution.topology.output_reuse_blockers] Mantle MUST fail closed before invoking `rustc` when prior cached Rust topology output evidence is stale or incomplete.

#### Scenario: Stale cached output blocks reuse

r[rust_package_planning.unit_execution.topology.output_reuse_blockers.stale]

- GIVEN a prior per-unit execution receipt exists under the execution output root
- WHEN a declared output artifact named by that receipt is missing, unreadable, digest-mismatched, or the receipt no longer matches the current explicit unit inputs
- THEN Mantle MUST return a structured stale-cache blocker before invoking `rustc` for that unit.
- AND Mantle MUST NOT silently fall back to Cargo or an unreviewed rebuild for that stale cached unit.
