# Release Provenance Specification

## Purpose

Publish release evidence bundles only after complete staging and verification through an atomic no-clobber commit.

## Requirements

### Requirement: Release bundle publication is atomic

r[mantle.release_provenance.bundle_publication.atomic_commit] Mantle MUST assemble a release bundle in a private sibling staging directory and make it visible at the final path only through one same-filesystem atomic no-clobber rename after successful verification.

#### Scenario: Complete staged bundle is published

r[mantle.release_provenance.bundle_publication.fixtures.positive]
- GIVEN all declared inputs are valid, the final destination is absent, and staging verification passes
- WHEN release bundle creation commits
- THEN the final path MUST appear as one complete canonical bundle
- AND CLI success MUST be emitted only after the atomic rename succeeds.

#### Scenario: Concurrent destination creation does not clobber

r[mantle.release_provenance.bundle_publication.fixtures.negative.race]
- GIVEN another process creates a file, symlink, or directory at the final destination after planning and before commit
- WHEN Mantle attempts publication
- THEN the no-clobber commit MUST fail
- AND the competing destination MUST remain unchanged.

### Requirement: Staging is complete and verified before commit

r[mantle.release_provenance.bundle_publication.staging_validation] Mantle MUST write the canonical manifest after all planned artifacts are staged and MUST run the production bundle verifier against the staging root before the stage becomes commit-eligible.

#### Scenario: Invalid staged artifact prevents publication

r[mantle.release_provenance.bundle_publication.fixtures.negative.verification]
- GIVEN a staged artifact is missing, stale, malformed, noncanonical, or inconsistent with proof or external-evidence linkage
- WHEN staging verification runs
- THEN commit eligibility MUST be denied with a deterministic phase diagnostic
- AND the final destination MUST remain absent.

### Requirement: Failed assembly does not poison the final path

r[mantle.release_provenance.bundle_publication.failure_isolation] Any pre-commit copy, hash, serialization, policy, verification, or cleanup error MUST leave an existing final destination untouched or an absent final destination absent; partial artifacts MUST remain confined to a Mantle-owned stage.

#### Scenario: Mid-assembly failure is retryable

r[mantle.release_provenance.bundle_publication.retry]
- GIVEN a named assembly phase fails after one or more artifacts have been staged
- WHEN the command returns and the operator retries with corrected inputs
- THEN the failed attempt MUST NOT leave a partial final bundle
- AND a fresh attempt MUST be able to stage, verify, and publish without manual deletion of final-path contents.

#### Scenario: Stale stage cleanup is ownership-bounded

r[mantle.release_provenance.bundle_publication.stale_stage]
- GIVEN a sibling staging directory remains after interruption
- WHEN retry or cleanup evaluates it
- THEN Mantle MAY remove or quarantine it only when a valid ownership marker and matching plan identity prove it is Mantle-owned staging state
- AND an unrecognized sibling path MUST remain untouched.

### Requirement: Atomic publication planning is pure

r[mantle.release_provenance.bundle_publication.boundary] Mantle MUST compute artifact layout, destination eligibility, deterministic assembly order, state transitions, and commit eligibility in a pure core, while filesystem observation, capability setup, staging writes, verification I/O, rename, cleanup, and rendering remain in the shell.

#### Scenario: Publication state machine is deterministic

r[mantle.release_provenance.bundle_publication.boundary.test]
- GIVEN normalized input facts, destination observations, assembly events, and verification results
- WHEN the publication core evaluates them
- THEN it MUST return deterministic next states and blockers without filesystem, environment, clock, process, or output effects.

### Requirement: Publication regression evidence covers failures

r[mantle.release_provenance.bundle_publication.validation] The change MUST test successful publication and retry plus injected copy, hash, manifest, verification, cleanup, destination-symlink, and commit-race failures through the production release creation path.

#### Scenario: Readers never observe a partial final bundle

r[mantle.release_provenance.bundle_publication.validation.visibility]
- GIVEN production creation is observed at every named assembly and commit phase
- WHEN positive and injected-failure fixtures run
- THEN the final path MUST be absent before commit and complete after commit
- AND no failure fixture MAY expose a partial final manifest or artifact tree.
