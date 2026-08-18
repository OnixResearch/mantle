# Rust Package Planning Specification

## Purpose

Adds rustc-dev-guide reference binding requirements for Mantle Rust planning, compiler-policy adapters, and source-built Rust provider evidence.

## Requirements

### Requirement: rustc-dev-guide reference map

r[rust_package_planning.rustc_dev_guide.reference_map] Mantle MUST maintain claim-bearing rustc-dev-guide references for Rust planning work that depends on rustc architecture details.

#### Scenario: Pinned references support planning claims

r[rust_package_planning.rustc_dev_guide.reference_map.pinned]

- GIVEN a Rust planning, compiler-policy, or Rust provider claim depends on rustc-dev-guide material
- WHEN Mantle records the evidence or design basis
- THEN the evidence MUST name the guide section, pinned guide revision or content digest, consuming Mantle surface, and bounded claim.
- AND moving `main` branch URLs MAY be used only as discovery links, not as claim-bearing evidence.

### Requirement: rustc-dev-guide claim boundaries

r[rust_package_planning.rustc_dev_guide.claim_boundaries] Mantle MUST keep rustc-dev-guide-backed architecture references separate from rustc semantic correctness claims.

#### Scenario: HIR and MIR references do not imply compiler correctness

r[rust_package_planning.rustc_dev_guide.claim_boundaries.non_claims]

- GIVEN Mantle uses HIR, MIR, or rustc_driver guide material to scope a planner or compiler-policy adapter
- WHEN Mantle emits receipts, reports, or release evidence
- THEN the evidence MUST preserve non-claims for program correctness, compiler correctness, full Cargo compatibility, release reproducibility, and bootstrap correctness unless separate accepted evidence exists.

### Requirement: Backend-facing rustc invocation evidence

r[rust_package_planning.rustc_dev_guide.backend_invocation] Mantle MUST make guide-backed backend, codegen, and linker assumptions explicit in Rust unit planning evidence.

#### Scenario: Reviewable rustc backend surface is preserved

r[rust_package_planning.rustc_dev_guide.backend_invocation.reviewable]

- GIVEN a Rust unit plan relies on rustc backend behavior for crate type, codegen backend, target triple, linker arguments, metadata, `--extern`, `--cfg`, or path remapping
- WHEN Mantle records the unit derivation or execution receipt
- THEN the receipt MUST bind the relevant arguments, toolchain identity, dependency artifacts, guide reference IDs, and output artifact digests.
- AND unsupported or unstated backend assumptions MUST fail closed before a Cargo-free execution claim is made.

### Requirement: Guide-scoped compiler-policy adapters

r[rust_package_planning.rustc_dev_guide.compiler_policy_adapter] Mantle MUST scope compiler-policy adapters that inspect HIR, MIR, or rustc_driver surfaces with explicit guide references and bounded claims.

#### Scenario: Compiler-policy receipt binds guide-backed inspection scope

r[rust_package_planning.rustc_dev_guide.compiler_policy_adapter.receipts]

- GIVEN a compiler-policy adapter inspects rustc internals or compiler IR
- WHEN Mantle invokes the adapter for a Rust unit
- THEN the compiler-policy receipt MUST bind adapter identity, selected rustc identity, policy digest, guide reference IDs, invocation status, waiver summary when present, and non-claims.
- AND policy success MUST claim only configured compiler-policy profile compliance unless separate evidence supports a stronger claim.

### Requirement: Guide-backed Rust provider patch plans

r[rust_package_planning.rustc_dev_guide.source_provider_patch_plan] Mantle MUST bind source-built Rust provider patch-plan operations to explicit rustc-dev-guide references when those operations rely on rustc internals.

#### Scenario: Source-provider patch frontier is reviewable

r[rust_package_planning.rustc_dev_guide.source_provider_patch_plan.frontier]

- GIVEN a Rust source-provider stage applies patches for rustc_driver, sysroot discovery, codegen backend layout, tool rlib lookup, or bootstrap workspace isolation
- WHEN Mantle records the patch plan or provider receipt
- THEN the evidence MUST name the operation, source anchor, stage, guide reference ID, input digest, output digest, and remaining non-claims.

### Requirement: rustc-dev-guide reference validation

r[rust_package_planning.rustc_dev_guide.final_validation] The change MUST include positive and negative fixtures plus focused validation before archive.

#### Scenario: Guide reference fixtures cover accepted and rejected claims

r[rust_package_planning.rustc_dev_guide.final_validation.fixtures]

- GIVEN positive fixtures with pinned guide references and negative fixtures with unpinned links, missing references, unsupported HIR/MIR semantic claims, or unstated backend assumptions
- WHEN focused validation runs
- THEN positive fixtures MUST pass and negative fixtures MUST fail closed with deterministic diagnostics.
