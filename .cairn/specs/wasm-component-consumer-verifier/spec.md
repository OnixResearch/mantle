# Wasm Component Consumer Verifier Specification

## Purpose

Defines the `wasm-component-consumer-verifier` capability.

## Requirements

### Requirement: Consumer verifier has a narrow versioned contract

r[mantle.wasm_consumer_verifier.contract] Mantle MUST publish a versioned consumer contract for materialization-bundle decoding, canonical identity validation, required-stage linkage, member-observation admission, ordered blockers, and report payloads without exposing build, scheduler, store, cache, or release authority.

#### Scenario: Consumer validates a complete bundle

- GIVEN a consumer supplies a supported bundle and complete bounded member observations
- WHEN contract validation runs
- THEN the verifier MUST return one deterministic decision under the declared schema
- AND it MUST NOT require a Mantle daemon, writable store, or build session.

#### Scenario: Consumer supplies an unsupported schema

- GIVEN a bundle uses an unknown schema or omits a required contract field
- WHEN contract validation runs
- THEN the verifier MUST reject it with a stable blocker before member files are trusted.

### Requirement: Verification decisions stay in a functional core

r[mantle.wasm_consumer_verifier.functional_core] Schema admission, canonical bundle identity, role and logical-path validation, stage-parent linkage, expected member facts, observation comparison, blocker ordering, and report construction MUST be pure deterministic logic over owned in-memory values and MUST support `no_std + alloc`.

#### Scenario: Equal facts are evaluated twice

- GIVEN two equal bundle and observation inputs
- WHEN the core evaluates both inputs
- THEN it MUST return equal decisions and report payloads
- AND it MUST perform no filesystem, store, process, network, environment, clock, runtime, or output effect.

### Requirement: File verification uses an explicit capability root

r[mantle.wasm_consumer_verifier.file_shell] The standard-library verifier MUST resolve the bundle and every declared member relative to one explicit capability root, MUST remeasure exact bytes and lengths under named bounds, and MUST reject absolute paths, parent traversal, symlinks, special files, duplicate paths, changed file types, and member substitution.

#### Scenario: Every member matches under the root

- GIVEN a complete bundle whose regular-file members resolve beneath the supplied root with matching lengths and BLAKE3 identities
- WHEN file verification runs
- THEN the shell MUST return complete member observations to the core.

#### Scenario: A member escapes or changes

- GIVEN a member path escapes the root, resolves through a symlink, changes type, exceeds a bound, or has different bytes
- WHEN file verification runs
- THEN the shell MUST fail before a passing materialized-byte report is emitted.

### Requirement: Structural and file verification remain distinct

r[mantle.wasm_consumer_verifier.layers] The verifier MUST identify which verification layers ran and MUST NOT label structural bundle validation as materialized-byte verification when required member observations are absent.

#### Scenario: Only parsed bundle data is available

- GIVEN a structurally valid bundle and no member root or byte observations
- WHEN structural verification runs
- THEN the report MAY record structural success
- AND materialized-byte status MUST remain blocked.

### Requirement: Consumer reports are bounded and safe

r[mantle.wasm_consumer_verifier.report] A verifier report MUST bind the schema, bundle identity, runtime-profile identity, verified member roles and BLAKE3 values, completed layers, bounded counts, blocker classes, and non-claims while excluding private absolute paths, payload bytes, credentials, environment values, and raw tool diagnostics.

#### Scenario: Passing report is rendered

- GIVEN complete passing structural and file decisions
- WHEN the shell renders a report
- THEN the report MUST contain only safe identities, counts, layer status, and non-claims.

#### Scenario: Raw path or payload enters a report

- GIVEN a proposed report contains a private root path or component payload bytes
- WHEN report admission runs
- THEN the report MUST be rejected or redacted before publication.

### Requirement: Published verifier has immutable consumer evidence

r[mantle.wasm_consumer_verifier.publication] Stable publication MUST bind one immutable source revision, documented public API, schema identity, feature model, exact validation commands, and independent consumer evidence without sibling-path dependencies.

#### Scenario: Publication has independent consumers

- GIVEN Kamacite and another consumer pass through the published facade from immutable source inputs
- WHEN publication readiness runs
- THEN Mantle MAY publish the verifier revision with its bounded evidence.

#### Scenario: Only sibling worktrees pass

- GIVEN consumer evidence depends on ambient sibling paths or mutable revisions
- WHEN publication readiness runs
- THEN publication MUST remain blocked.

### Requirement: Positive and negative fixtures cover the boundary

r[mantle.wasm_consumer_verifier.fixtures] The focused verifier rail MUST include positive complete-bundle and independent-consumer fixtures plus negative schema, identity, stage-link, role, path, symlink, file-type, length, byte-drift, bound, missing-member, report-leak, and overclaim fixtures.

#### Scenario: Fixture matrix runs

- GIVEN the consumer facade and shell are proposed for publication
- WHEN the focused rail runs
- THEN every positive fixture MUST pass
- AND every negative fixture MUST fail at its declared boundary.

### Requirement: Verifier evidence preserves non-claims

r[mantle.wasm_consumer_verifier.nonclaims] Passing verifier evidence MUST NOT claim source trust, compiler correctness, component behavior, runtime isolation, authority, reproducibility, deployment safety, or release eligibility.

#### Scenario: Consumer promotes bundle validity to correctness

- GIVEN a consumer labels a passing bundle report as component correctness or runtime authority
- WHEN non-claim validation runs
- THEN the evidence MUST fail with a deterministic overclaim blocker.
