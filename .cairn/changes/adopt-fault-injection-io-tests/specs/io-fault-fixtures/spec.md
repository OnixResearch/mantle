# IO Fault Fixtures Specification

## Purpose

Give Mantle deterministic, annotated, test-only injection of `io::Error` so the named build/store I/O failure paths are provable without a damaged store.

## Requirements

### Requirement: Injection is test-only and pinned [r[mantle.io_fault.fixtures]]

The repository MUST confine `fault-injection` to dev-dependencies of the shell test target. The pinned revision MUST be recorded as transport `crates.io`, plane `validation`.

#### Scenario: Runtime code stays unaware

- GIVEN the workspace dependency graph after adoption
- WHEN dependency classification runs
- THEN `fault-injection` MUST appear only on dev edges

#### Scenario: Store boundary sites are wrapped

- GIVEN the shell's store blob read, NAR stream write, post-build action-result record pre-link fsync, and action-result cache write sites
- WHEN the test target builds
- THEN each site MUST pass through `fallible!`

### Requirement: Injection is deterministic [r[mantle.io_fault.determinism]]

Fixtures MUST store an explicit counter value before acting and MUST restore the default afterward. `SLEEPINESS` MUST stay zero.

#### Scenario: Fixture fires at a chosen site

- GIVEN a fixture stores a counter value that reaches zero at the post-build action-result record pre-link fsync
- WHEN publication attempts to commit the cache record
- THEN the fsync MUST return an injected `io::Error`
- AND the store adapter MUST retain the `action-result-publication-sync-temp` diagnostic; ordinary publication remains diagnostic while CA-required realisation MUST fail closed as `ca-realisation-untrusted`

#### Scenario: Random delays stay disabled

- GIVEN any fixture or CI configuration
- WHEN the test suite runs
- THEN `SLEEPINESS` MUST be zero

### Requirement: Injection claims nothing beyond unit coverage [r[mantle.io_fault.boundary]]

Documentation MUST state that injection coverage grants no sandbox-hermeticity, store-correctness, or release-eligibility claim.

#### Scenario: Over-claim rejected

- GIVEN documentation claims store correctness from injection fixtures
- WHEN boundary verification runs
- THEN the claim MUST fail verification

### Requirement: Failure paths stay covered [r[mantle.io_fault.verification]]

Positive, negative, and boundary fixtures MUST cover every wrapped site and the annotation contract.

#### Scenario: Complete matrix passes

- GIVEN fixtures for the unchanged cycle, each injected site, and annotation boundaries
- WHEN package, workspace, Clippy, Cairn, and Nix checks run
- THEN each injected site MUST produce its declared classification
- AND the unchanged cycle MUST behave as before
