# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Rust unit execution from explicit derivation plans

r[rust_package_planning.unit_execution] Mantle MUST execute supported Rust package units from explicit `unit_derivation_graph` receipt nodes rather than invoking Cargo as a hidden build orchestrator.

#### Scenario: Supported unit executes from receipt material

r[rust_package_planning.unit_execution.supported_unit]

- GIVEN a `unit_derivation_graph` is ready and contains a supported `lib` or `bin` unit node
- WHEN Mantle executes that unit
- THEN Mantle MUST invoke `rustc` using the node's explicit arguments, environment, declared inputs, source-closure digest, dependency artifacts, host artifacts, and declared outputs.
- AND Mantle MUST NOT consult Cargo target directories, registry caches, git checkouts, or build orchestration as undeclared execution inputs.

#### Scenario: Bounded Cargo-free execution claim

r[rust_package_planning.unit_execution.bounded_claim]

- GIVEN Mantle successfully executes a supported explicit Rust unit
- WHEN Mantle reports the result
- THEN the report MAY claim Cargo-free execution for that explicit unit node only.
- AND the report MUST NOT claim full Cargo compatibility, doctest/test/example support, native-link probing correctness, rustc/compiler correctness, or full bootstrap correctness unless separate evidence exists.

### Requirement: Rust unit execution receipts

r[rust_package_planning.unit_execution_receipts] Mantle MUST emit deterministic receipts for Rust unit execution results.

#### Scenario: Output identity is receipt-bound

r[rust_package_planning.unit_execution_receipts.output_identity]

- GIVEN a Rust unit execution finishes successfully
- WHEN Mantle records the execution receipt
- THEN the receipt MUST bind the unit identity, source closure digest, dependency artifact digests, host artifact digests, toolchain identity, `rustc` argument digest, declared output paths, output artifact digests, execution status, and rebuild or reuse reason.
- AND the receipt MUST be deterministic across repeated executions with equivalent declared inputs.

#### Scenario: Failed execution preserves reviewable diagnostics

r[rust_package_planning.unit_execution_receipts.failure]

- GIVEN `rustc` execution fails for a supported unit
- WHEN Mantle records the failure
- THEN the receipt MUST include deterministic failure class, unit identity, input digest summary, and redacted diagnostics sufficient for review.
- AND the receipt MUST NOT include secrets, absolute temporary paths as hash material, or ambient environment dumps.

### Requirement: Rust unit execution fail-closed blockers

r[rust_package_planning.unit_execution_blockers] Mantle MUST reject Rust unit execution when required explicit material is absent or inconsistent.

#### Scenario: Missing execution material blocks before rustc

r[rust_package_planning.unit_execution_blockers.missing_material]

- GIVEN a supported Rust unit requires source closure material, dependency artifacts, host artifacts, declared output paths, or toolchain identity
- WHEN any required material is missing, unreadable, inconsistent with its digest, or not represented in the unit derivation receipt
- THEN Mantle MUST fail before invoking `rustc` with a deterministic blocker identifying the missing material class.
- AND Mantle MUST NOT search ambient Cargo caches or target directories to repair the missing material.

#### Scenario: Unsupported execution boundary remains explicit

r[rust_package_planning.unit_execution_blockers.unsupported_boundary]

- GIVEN a Rust unit requires execution behavior outside Mantle's supported subset
- WHEN Mantle evaluates the unit for execution
- THEN Mantle MUST emit a deterministic unsupported-boundary blocker rather than invoking Cargo or claiming a successful native execution.
