## ADDED Requirements

### Requirement: Wrapperless source-root fixed-point proof

r[rust_package_planning.wrapperless_source_root_fixed_point] Mantle MUST run the Cargo-free fixed-point command from declared source-root or toolchain-closure inputs without requiring a caller-created Nix, rustup, or otherwise untracked rustc wrapper.

#### Scenario: command-owned normalization is receipt-bound

GIVEN an operator launches the Cargo-free fixed-point command with a declared source-root provider or toolchain-closure manifest
WHEN Mantle prepares rustc, linker, archive, or helper-tool compatibility normalization
THEN every generated wrapper or normalization rule MUST be derived from declared closure members and written as receipt-bound stage material.
AND the proof bundle MUST record each generated file path, BLAKE3 digest, selected input digest, and normalization rule.

#### Scenario: external wrappers fail closed

GIVEN `RUSTC`, PATH, or an explicit proof argument points at a rustc/linker wrapper that is not a declared closure member
WHEN the fixed-point command evaluates preflight
THEN Mantle MUST fail before stage topology execution with an external-wrapper blocker.
AND it MUST NOT claim wrapperless, source-root, or source-built toolchain closure proof success.

#### Scenario: both stages enforce the same policy

GIVEN stage1 and stage2 fixed-point builds are launched
WHEN Mantle constructs their execution environments
THEN both stages MUST use the same closure policy digest, guard policy digest, and command-owned normalization plan digest unless a deterministic policy-mismatch blocker is reported.
AND stage2 MUST NOT inherit undeclared host wrappers from the stage1 launcher environment.

#### Scenario: ambient Nix and rustup remain guarded

GIVEN the fixed-point command is running under a host that has Nix, rustup, Cargo, or profile toolchains on ambient PATH
WHEN any stage attempts to resolve a protected tool
THEN resolution MUST use only declared closure members or command-owned wrapper material.
AND undeclared Nix, rustup, Cargo, or ambient toolchain resolution MUST fail with a deterministic host-tool-leakage blocker before a proof claim.

#### Scenario: summary states the remaining frontier

GIVEN wrapperless source-root fixed-point execution completes, blocks, or mismatches
WHEN Mantle writes the proof summary
THEN the summary MUST identify wrapperless status, source-root closure status, per-stage policy digests, fixed-point binary digests when present, exact blocker class when blocked, and bounded non-claims.
AND a successful stage1==stage2 comparison MUST NOT claim release reproducibility, full Cargo compatibility, compiler correctness, or a zero-seed bootstrap root unless separate evidence exists.
