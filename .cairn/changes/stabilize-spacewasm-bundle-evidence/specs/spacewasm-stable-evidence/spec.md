# SpaceWasm Stable Evidence Specification

## ADDED Requirements

### Requirement: Stable test facts have an explicit contract

r[mantle.spacewasm_stable_evidence.contract] Mantle MUST define a versioned contract for stable test facts, canonical encoding, and BLAKE3 identity.

The contract MUST bind source, dependency closure, toolchain, target, features, exact command arguments, suite identity, expected test inventory, and observed outcomes. It MUST define named input, record, output, and execution bounds in typed Nickel policy. Runtime code MUST consume a checked deterministic export.

Only explicitly declared presentation differences MAY leave the stable projection unchanged. Elapsed times and completion order MUST NOT become stable identity inputs. Changed test identity, outcome, selection, or executable identity MUST remain detectable.

#### Scenario: Presentation differs but test facts agree
- GIVEN complete admitted reports with identical test facts and different elapsed times or completion order
- WHEN Mantle constructs their stable projections
- THEN their canonical bytes and BLAKE3 identities MUST match.

#### Scenario: A required fact changes
- GIVEN a changed test name, outcome, filter, feature, command argument, source, or executable identity
- WHEN Mantle evaluates the reports
- THEN Mantle MUST reject the mismatch or produce a distinct admitted identity, never erase the difference as presentation.

### Requirement: Raw diagnostics retain a separate identity

r[mantle.spacewasm_stable_evidence.diagnostics] Mantle MUST retain exact raw diagnostic bytes through an explicit run-evidence contract separate from the stable bundle identity.

The run record MUST bind the stable bundle, exact command and inputs, process outcome, capture completeness, and raw BLAKE3 identities. The stable bundle MUST NOT transitively bind variable raw hashes, run identifiers, timestamps, or physical capture paths. The shell MUST retain diagnostics under declared authority and retention policy without writing a nondeterministic secondary Nix output.

Publication MUST fail when required diagnostic capture or retention fails. Normalized text MUST NOT claim raw-byte identity. Historical reports MUST remain unchanged.

#### Scenario: Two successful executions have different raw logs
- GIVEN matching stable facts and complete retained raw captures from separate executions
- WHEN Mantle records their evidence
- THEN the run records MUST preserve their distinct raw identities and bind the same stable bundle identity.

#### Scenario: Raw evidence is incomplete
- GIVEN a missing, truncated, unreadable, or unretained required capture
- WHEN publication evaluates evidence completeness
- THEN Mantle MUST report the exact blocker and MUST NOT emit a successful publication receipt.

### Requirement: Invalid evidence cannot become success

r[mantle.spacewasm_stable_evidence.denial] Mantle MUST reject evidence with missing or duplicate required tests, unknown formats, contradictory summaries, unsupported statuses, invalid encoding, exceeded bounds, or incomplete process outcomes.

Nonzero exit, signal termination, timeout, and cancellation MUST remain failures even when stdout contains a passing summary. Skipped, unavailable, unsupported, or filtered required tests MUST NOT become passed. The projection MUST preserve required failure evidence and MUST NOT deduplicate conflicting records or silently ignore unknown input.

#### Scenario: Complete successful suite is admitted
- GIVEN one admitted result for every required test and a complete successful process outcome
- WHEN evidence admission runs
- THEN Mantle MAY emit stable successful test facts.

#### Scenario: Passing text conflicts with the process outcome
- GIVEN a passing summary paired with a nonzero exit, timeout, signal, or cancellation
- WHEN evidence admission runs
- THEN Mantle MUST reject successful qualification.

#### Scenario: Test inventory or syntax is invalid
- GIVEN missing, duplicate, filtered, malformed, unknown, oversized, or contradictory records
- WHEN evidence admission runs
- THEN Mantle MUST reject successful qualification with a bounded diagnostic.

### Requirement: Stable report meaning remains in the functional core

r[mantle.spacewasm_stable_evidence.boundary] Pure report admission, inventory comparison, canonical ordering, identity construction, and rejection decisions MUST remain in `crunch-spacewasm-core` over explicit bounded in-memory facts.

The `crunch-spacewasm` and Nix shells MUST own process execution, clocks, capture, filesystem authority, retention, and publication. Dependencies MUST point inward. The implementation MUST reuse compatible existing components through published pinned contracts and MUST NOT add generic internal-function ports or sibling-worktree dependencies.

#### Scenario: Core replay has no host effects
- GIVEN identical admitted profile and result facts
- WHEN the core evaluates them under different host environments
- THEN decisions and canonical bytes MUST match without filesystem, process, network, clock, or environment access.

#### Scenario: Diagnostic storage fails
- GIVEN a valid core projection and a shell storage error
- WHEN the shell attempts publication
- THEN the shell MUST report failure without changing the core result into a storage-success claim.

### Requirement: Rebuild evidence reruns variable producers

r[mantle.spacewasm_stable_evidence.rebuild] Mantle MUST prove stable bundle reproducibility across fresh executions of every report-producing dependency under identical admitted inputs.

The proof MUST record exact source and derivation identities, command results, per-member measurements, manifest identity, bundle identity, and retained raw captures. A cache hit, reused report directory, or bundle-only rebuild MUST NOT count as repeated producer execution. Comparisons MUST include every required member and parent edge.

#### Scenario: The complete producer graph repeats
- GIVEN separate scratch roots and fresh executions of report producers under identical admitted inputs
- WHEN Mantle compares the resulting stable bundles
- THEN manifest bytes, bundle identity, and every required member MUST match.

#### Scenario: Evidence changes or execution is reused
- GIVEN a tampered member, changed result, missing parent edge, or reused report-producing output
- WHEN the reproducibility gate runs
- THEN the gate MUST reject the reproducibility claim.

### Requirement: Consumer migration preserves independent admission

r[mantle.spacewasm_stable_evidence.handoff] Mantle MUST publish an immutable reviewed repair revision and an explicit compatibility contract before downstream admission changes.

Changed bundle membership, report roles, or identity semantics MUST use an explicit version transition rather than reinterpret existing receipts. Maintainers MUST preserve accepted materialization requirements or supply reviewed delta modifications before implementation. ChaosControl MUST independently remeasure the published candidate and retain manifest, member, wrong-pin, tamper, and overclaim denials.

Producer evidence MUST NOT claim consumer runtime admission, Campaign adoption, SpaceWasm correctness, WebAssembly conformance, compiler correctness, sandbox effectiveness, or release eligibility.

#### Scenario: The consumer imports a verified candidate
- GIVEN an immutable producer revision with complete repeatability and compatibility evidence
- WHEN ChaosControl evaluates the candidate under its own admission profile
- THEN the consumer MAY accept only its separately verified bounded claims.

#### Scenario: A consumer bypasses a mismatch
- GIVEN a digest refresh without producer repeatability evidence or a disabled member check
- WHEN closeout evaluates the consumer handoff
- THEN closeout MUST remain blocked.
