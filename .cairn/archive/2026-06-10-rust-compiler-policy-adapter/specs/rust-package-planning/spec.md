## ADDED Requirements

### Requirement: Rust compiler policy adapter

r[rust_package_planning.compiler_policy_adapter] Mantle MUST model Rust compiler-policy enforcement as an explicit adapter at the Rust unit invocation boundary, separate from the base `rustc` tool identity and separate from rule-provider implementation details.

#### Scenario: Direct Rust unit invocation is adapted

r[rust_package_planning.compiler_policy_adapter.invocation]

- GIVEN a supported Rust unit is ready for direct execution from explicit `unit_derivation_graph` material
- WHEN an operator selects a compiler-policy adapter
- THEN Mantle MUST resolve the base rustc path, unit arguments, unit environment, adapter executable, adapter environment, and adapter identity before invoking the compiler.
- AND Mantle MUST keep filesystem checks, digesting, command construction, and process execution in the imperative shell rather than in pure planning logic.

#### Scenario: Octet adapter remains provider-owned

r[rust_package_planning.compiler_policy_adapter.octet]

- GIVEN the selected compiler-policy provider is Octet
- WHEN Mantle prepares a Rust unit invocation
- THEN Mantle MUST consume Octet-provided adapter artifacts or a provider manifest rather than importing Octet lint implementation internals.
- AND Mantle MUST treat Octet as the authority for lint names, lint levels, standards checks, suppression validation, and diagnostic semantics.

#### Scenario: Operator modes bound enforcement strength

r[rust_package_planning.compiler_policy_adapter.cli]

- GIVEN an operator invokes Rust-plan execution
- WHEN no compiler-policy adapter is selected
- THEN Mantle MUST preserve the current plain rustc behavior and make no Octet compliance claim.
- WHEN audit, deny, or required policy modes are selected
- THEN Mantle MUST record the selected mode and apply its fail-open or fail-closed behavior deterministically.

### Requirement: Compiler policy identity bounds Rust output reuse

r[rust_package_planning.compiler_policy_adapter.cache_identity] Mantle MUST include compiler-policy adapter identity in Rust unit output reuse and execution receipt identity whenever a compiler-policy adapter is selected.

#### Scenario: Raw-rustc output cannot satisfy required policy

r[rust_package_planning.compiler_policy_adapter.cache_identity.raw_rustc_rejected]

- GIVEN a Rust unit output was previously produced without a compiler-policy adapter
- WHEN the same unit is later executed with an Octet-required compiler-policy mode
- THEN Mantle MUST reject reuse of the prior raw-rustc output.
- AND Mantle MUST rebuild or fail closed using the required adapter rather than treating source, dependency, and rustc-argument equality as sufficient.

#### Scenario: Adapter artifacts are receipt-bound

r[rust_package_planning.compiler_policy_adapter.receipts]

- GIVEN a Rust unit execution uses a compiler-policy adapter
- WHEN Mantle records a successful, failed, blocked, or reused execution receipt
- THEN the receipt MUST include the adapter kind, mode, provider-manifest digest when present, driver digest when present, lint-library digest when present, config digest when present, standards-policy digest when present, and a waiver summary when present.
- AND the receipt MUST keep policy-profile compliance claims separate from program-correctness or full-architecture-proof claims.

### Requirement: Compiler policy required mode fails closed

r[rust_package_planning.compiler_policy_adapter.fail_closed] Mantle MUST fail before accepting a Rust unit output when required compiler-policy material is absent, inconsistent, or unable to run.

#### Scenario: Missing adapter material blocks before output acceptance

r[rust_package_planning.compiler_policy_adapter.fail_closed.missing_material]

- GIVEN Octet-required mode is selected
- WHEN the adapter driver, lint library, config, provider manifest, standards policy artifact, or declared digest is missing or mismatched
- THEN Mantle MUST emit a deterministic compiler-policy blocker.
- AND Mantle MUST NOT fall back to plain rustc, weaken the policy mode, or reuse unadapted outputs.

#### Scenario: Standards gate is separate and receipt-bound

r[rust_package_planning.compiler_policy_adapter.standards_gate]

- GIVEN the selected policy profile includes Octet FCIS source-shape standards
- WHEN Mantle accepts a topology-level Rust build result
- THEN Mantle MUST run or verify the declared standards gate and bind its policy artifact, selected scope, pass/fail status, and waiver summary into the topology evidence.
- AND Mantle MUST NOT describe standards-gate success as a formal proof of program correctness.

### Requirement: Compiler policy claims remain bounded

r[rust_package_planning.compiler_policy_adapter.non_claims] Mantle MUST report compiler-policy enforcement as compliance with a named, hashed policy profile rather than as a proof of program correctness or full architecture correctness.

#### Scenario: Compliance wording names explicit scope

r[rust_package_planning.compiler_policy_adapter.non_claims.scope]

- GIVEN Rust unit or topology execution succeeds with a compiler-policy adapter
- WHEN Mantle renders human or JSON evidence
- THEN the evidence MAY claim that the selected unit or topology was accepted under the configured compiler-policy profile.
- AND the evidence MUST NOT claim semantic correctness, memory safety beyond Rust's guarantees, complete FCIS architecture, full Cargo compatibility, or absence of all defects unless separate evidence is present.
