## ADDED Requirements

### Requirement: Provider-backed source-built Rust closure status

r[rust_package_planning.source_built_toolchain_closure.provider_status] Mantle MUST promote a validated source-built Rust provider into the Cargo-free proof `source_built_toolchain_closure` status instead of reporting the stale `not-source-built-toolchain-closure` non-claim.

#### Scenario: Validated provider supplies closure status

GIVEN a Cargo-free one-shot or fixed-point proof is launched with `--rust-source-provider` and without an explicit `--toolchain-closure` manifest
WHEN Mantle validates the provider metadata and uses the provider Rust compiler for topology execution
THEN the proof summary MUST record `source_built_toolchain_closure.status = provided`, `claim = true`, and the provider policy digest.
AND the proof summary MUST NOT include `not-source-built-toolchain-closure` in `non_claims`.

#### Scenario: Absent provider keeps the non-claim

GIVEN a Cargo-free one-shot or fixed-point proof is launched without `--rust-source-provider` and without an explicit `--toolchain-closure` manifest
WHEN Mantle writes preflight or summary evidence
THEN it MUST keep `source_built_toolchain_closure.status = not-provided`, `claim = false`, and `not-source-built-toolchain-closure`.

#### Scenario: Explicit closure manifest remains authoritative

GIVEN a Cargo-free proof is launched with both `--rust-source-provider` and `--toolchain-closure`
WHEN Mantle validates or enforces the toolchain closure
THEN the explicit closure manifest MUST remain authoritative for `source_built_toolchain_closure` status and policy digest.
AND Mantle MUST still fail on undeclared toolchain input leakage or stage1/stage2 policy digest mismatch.

#### Scenario: Broader proof bounds remain visible

GIVEN a provider-backed Cargo-free fixed-point proof succeeds
WHEN Mantle writes non-claim evidence
THEN it MUST keep unrelated proof bounds including not release reproducibility and not full Cargo compatibility until separate evidence retires those bounds.
