### Requirement: Rust target-unit topology execution

r[rust_package_planning.unit_execution.target_topology] Mantle MUST execute a bounded target-only Rust unit topology from explicit unit derivation graph evidence without invoking Cargo as the build orchestrator.

#### Scenario: Executes supported target units in dependency order

r[rust_package_planning.unit_execution.target_topology.executes]

- GIVEN `unit_derivation_graph` is ready and contains supported target `lib`/`bin` units whose dependency artifacts are produced by supported target `lib` units in the same graph
- WHEN target topology execution is requested
- THEN Mantle MUST execute producer units before consumers using the explicit derivation args/env.
- AND Mantle MUST rebind produced `.rlib` artifacts into downstream dependency artifact, input, and `--extern` surfaces before invoking each consumer `rustc`.
- AND Mantle MUST NOT invoke Cargo as the build orchestrator for the topology execution.

### Requirement: Rust target-unit topology blockers

r[rust_package_planning.unit_execution.target_topology.blockers] Mantle MUST fail closed with deterministic blockers for unsupported or incomplete topology shapes before invoking an affected consumer `rustc`.

#### Scenario: Blocks unsupported target topology shapes

r[rust_package_planning.unit_execution.target_topology.blockers.unsupported]

- GIVEN the graph is not ready, requires host/proc-macro/build-script artifacts, contains missing producer libs, cycles, missing produced artifacts, or unsupported target modes/kinds
- WHEN target topology execution is requested
- THEN Mantle MUST emit a deterministic topology blocker receipt.
- AND Mantle MUST NOT claim successful target-topology execution for the unsupported shape.

### Requirement: Rust target-unit topology receipts

r[rust_package_planning.unit_execution_receipts.target_topology] Mantle MUST emit a target topology execution receipt that preserves ordered per-unit execution evidence and deterministic blocker evidence.

#### Scenario: Receipt preserves ordered topology evidence

r[rust_package_planning.unit_execution_receipts.target_topology.ordered]

- GIVEN target topology execution is requested
- WHEN Mantle reports the result
- THEN the receipt MUST include ordered unit execution receipts, output artifact BLAKE3 digests, dependency artifact BLAKE3 digests for consumers, a bounded target-only claim, an optional blocker, and a stable receipt hash.

### Requirement: Rust target-unit topology CLI receipts

r[rust_package_planning.unit_execution_receipts.target_topology.cli] Mantle MUST expose target topology execution as reviewable `rust-plan` CLI JSON evidence.

#### Scenario: CLI emits captured plan plus topology execution receipt

r[rust_package_planning.unit_execution_receipts.target_topology.cli.json]

- GIVEN `mantle rust-plan` captures a ready target-only unit graph
- WHEN the target topology CLI flag is requested with an explicit execution output root
- THEN Mantle MUST emit JSON containing the captured Rust plan receipt and the target topology execution receipt.
