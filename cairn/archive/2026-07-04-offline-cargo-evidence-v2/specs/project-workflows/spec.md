## ADDED Requirements

### Requirement: Offline Cargo evidence is digest-bound

r[project_workflows.offline_cargo_digest_bound_evidence] Mantle MUST emit and report versioned offline Cargo package evidence that binds the Cargo action to declared offline inputs by digest. The evidence MUST record the package source identity, Cargo.lock BLAKE3 digest, optional vendored dependency source identity, selected Rust/toolchain inputs, target, profile, Cargo command shape, network policy result, output identity, and bounded non-claims. Malformed or stale evidence MUST be diagnosed rather than silently promoted to a successful offline Cargo evidence claim.

#### Scenario: successful offline Cargo build reports digest-bound evidence

GIVEN a Mantle project builds a Rust package through `mantle.offlineCargoPackage`
AND the build uses declared source, lockfile, optional vendor, Rust toolchain, seed toolchain, and musl inputs
WHEN Mantle writes the sidecar and renders a JSON build report
THEN the report MUST include digest-bound `cargo-inside-mantle-sandbox` evidence for that output
AND the evidence MUST include non-claims for Cargo-free execution, full Cargo compatibility, compiler correctness, release reproducibility, and bootstrap correctness.

#### Scenario: vendor identity is included when declared

GIVEN an offline Cargo package declares `vendor_src`
WHEN the package writes build evidence
THEN the evidence MUST include the vendored dependency source role, path or source-state identity, and BLAKE3 digest or explicit narrower identity class
AND the report MUST distinguish package source, vendor source, and toolchain inputs.

#### Scenario: malformed sidecar is diagnostic

GIVEN an output contains an offline Cargo evidence sidecar with malformed JSON, unsupported schema, wrong claim class, missing mandatory fields, invalid digest syntax, or stale expected input digest
WHEN Mantle builds or reports that output
THEN Mantle MUST emit a deterministic diagnostic for the evidence failure
AND it MUST NOT report digest-bound offline Cargo evidence for that output.

#### Scenario: evidence does not strengthen Cargo claims

GIVEN offline Cargo evidence is digest-bound and valid
WHEN Mantle docs, reports, tasks, or release material cite it
THEN the claim MUST remain limited to the declared Cargo action running inside Mantle's sandbox with the recorded offline inputs
AND it MUST NOT claim Cargo-free execution, full Cargo compatibility, compiler correctness, release reproducibility, or bootstrap correctness without separate evidence.
