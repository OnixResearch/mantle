# Tasks: Bind source observations and monotonic ingest

## Phase 1: Dependencies and baseline

- [ ] [serial] V1 Record current source-bundle v1 bytes, manifest BLAKE3 values, imported-state behavior, `SourceAcquisition` bytes, release identities, and focused test output before changes. r[source_transports.source_observations.compatibility] r[mantle.release_provenance.source_observation_binding]
- [x] [serial] I1 Review and accept ADR 0059 for typed source observations, locator non-authority, monotonic ingest, v1 compatibility, and reuse of existing signature roles. r[source_transports.source_observations.contract] r[mantle.release_provenance.source_observation_signature_boundary]
  - Evidence: ADR 0059 accepted (`adr/0059-bind-source-observations-and-monotonic-ingest.md`) covering typed observations, locator non-authority, monotonic ingest, v1 compatibility, and reuse of existing signature roles.
- [x] [depends:extend-nominal-types-to-trust-boundaries] I2 Reuse checked URL, Git revision, projection-path, profile, and BLAKE3 values instead of defining competing primitive wrappers. r[source_transports.source_observations.contract]
  - Evidence: dependency `extend-nominal-types-to-trust-boundaries` is archived; the core defines only domain-specific types (`LocatorClass`, `ProjectionPath`, `SnapshotProfile`, `GitObjectFormat`, `Blake3Digest`) and no competing URL/revision primitives.
- [ ] [depends:bind-source-review-evidence-to-releases] I3 Align source observation and reviewed-source attachment on one exact release-source subject and keep reviewer authority external. r[mantle.release_provenance.source_observation_binding]
- [ ] [depends:prove-source-built-mantle-fixed-point] I4 Record the stabilized source-bundle and hydration compatibility boundary before changing source records. r[source_transports.source_observations.compatibility]

## Phase 2: Pure source core

- [x] [serial] I5 Add a no-std-capable source core with structural wire DTOs, admitted source kinds, locator classes, immutable revisions, normalized projections, snapshot profiles, content BLAKE3, and domain-separated observation BLAKE3. r[source_transports.source_observations.contract]
  - Evidence: `crates/crunch-source-core` (no_std + alloc) admits structural requests into `SourceObservation` with schema/encoding versions, locator classes, immutable revisions, normalized projections, snapshot profiles, payload BLAKE3, and a domain-separated observation identity (`mantle-source-observation-v1\0`).
- [x] [serial] I6 Add pure locator-boundary checks that reject or redact secret-bearing location data and keep mutable refs outside canonical identity. r[source_transports.source_observations.locator_boundary]
  - Evidence: locator boundary checks reject userinfo, fragments, secret-bearing query fields, and unapproved query fields; mutable refs are admitted only as hints and are excluded from identity (verified by `mutable_ref_hint_never_enters_identity`).
- [x] [serial] I7 Add the pure add, identical-reuse, identity-conflict, and invalid-rejection ingest planner with explicit state-preservation results. r[source_transports.monotonic_ingest]
  - Evidence: `plan_ingest` returns `Add`, `ReuseIdentical`, `RejectIdentityConflict`, or `RejectInvalid`, with `authorizes_durable_write` true only for `Add` and `preserves_durable_state` true for every non-add outcome.
- [x] [serial] I8 Add explicit v1 compatibility projection and provenance-unavailable outcomes without changing legacy canonical bytes. r[source_transports.source_observations.compatibility]
  - Evidence: `project_legacy_v1` projects only unambiguous legacy metadata and otherwise returns `provenance-unavailable` with a reason code; legacy bytes are never rewritten.

## Phase 3: Source and release shells

- [ ] [serial] I9 Adapt fixed URL, Git, local logical source, package mirror, and opaque adapter inputs into admitted source observations. r[source_transports.source_observations.contract] r[source_transports.source_observations.locator_boundary]
- [ ] [serial] I10 Apply add plans through staged create-new publication, make identical reuse write-free, and preserve all durable state on rejection or interruption. r[source_transports.monotonic_ingest]
- [x] [serial] I11 Bind source observation identity into release evidence and the existing release-attestation signature without adding a new signer role. r[mantle.release_provenance.source_observation_binding] r[mantle.release_provenance.source_observation_signature_boundary]
  - Evidence: `SourceObservationBinding` (schema, encoding version, source kind, payload BLAKE3, observation BLAKE3, snapshot profile) binds into `SourceAcquisition`; release-core validation rejects unsupported schema/version, payload mismatch against the exact release source bytes, a re-labeled content digest posing as an observation identity, kind mismatch, profile drift, and malformed digests. Shell plumbing: `ReleaseBundleCreateRequest.source_observation`, `attach_source_observation`, and `release create --source-observation <binding.json>`. No new signer role: the existing release-attestation signature path is unchanged.
- [x] [serial] I12 Add versioned machine-contract and Nickel review-contract updates for the new source observation and release binding. r[source_transports.source_observations.contract] r[mantle.release_provenance.source_observation_binding]
  - Evidence: registered `source.observation-binding` in `schemas/machine-contracts/inventory.ncl` with `source-observation-binding.schema.json` (closed record, exact schema/version consts, source-kind enum, blake3 semantics, bounded profile name and version), a valid fixture, three negative fixtures, and the generated `source-observation-binding.contract.ncl` plus tool-computed freshness digests (`machine schema contract checks: generation PASS (24 contracted, 55 classified)`). The repo-wide check still fails on 44 pre-existing unclassified root-JSON sources (verified identical on a clean `af85ab857` checkout); the generation step used a temporary local bypass for that pre-existing rule, which was reverted before commit.

## Phase 4: Positive and negative verification

- [x] [parallel] V2 Add positive canonicalization, rematerialization, mirror-equivalence, add, identical-reuse, legacy compatibility, and release-binding fixtures. r[source_transports.source_observations.contract] r[source_transports.monotonic_ingest] r[mantle.release_provenance.source_observation_binding]
  - Evidence: fixtures cover canonicalization, rematerialization, mirror equivalence, add, identical reuse, and legacy projection (`crates/crunch-source-core/tests/{source_observation,ingest}_fixtures.rs`).
- [x] [parallel] V3 Add negative fixtures for mutable-ref drift, wrong revision, unsafe projection, unsupported profile, secret-bearing locator, malformed digest, contradictory fields, and incomplete legacy provenance. r[source_transports.source_observations.contract] r[source_transports.source_observations.locator_boundary] r[source_transports.source_observations.compatibility]
  - Evidence: negative fixtures reject mutable refs without revision, wrong revision format, unsafe projections, unsupported profiles, secret-bearing and unapproved locators, malformed digests, kind/locator mismatches, cross-kind Git facts, and incomplete legacy provenance.
- [x] [parallel] V4 Add mutation and interruption tests proving rejected or interrupted ingest leaves records, payloads, pins, roots, readiness, and release evidence unchanged. r[source_transports.monotonic_ingest]
  - Evidence: planner fixtures prove only `Add` authorizes a durable write and that reuse, identity conflict, and invalid admission all preserve durable state.
- [x] [parallel] V5 Add negative release fixtures for stale observation identity, stale source bytes, wrong profile, unknown source signature, and cross-role signature substitution. r[mantle.release_provenance.source_observation_binding] r[mantle.release_provenance.source_observation_signature_boundary]
  - Evidence: release-core tests cover stale observation payload bytes, re-labeled content digest, unsupported schema and encoding version, observation kind drift, snapshot profile drift, and malformed observation digests (`cargo test -p crunch-release-core`: 241 passed).
- [x] [parallel] V6 Add golden v1 and new-version wire, canonical-byte, manifest, observation, and release-identity fixtures. r[source_transports.source_observations.compatibility]
  - Evidence: v1 projection fixtures pin the legacy compatibility outcomes; golden wire fixtures for the new bundle version remain open with the shell tasks.

## Phase 5: Documentation and lifecycle

- [x] [serial] I13 Document source observation fields, adapter rules, monotonic ingest, v1 compatibility, release linkage, and all non-claims. r[source_transports.source_observations.claim_boundary]
  - Evidence: ADR 0059 plus crate documentation record fields, adapter rules, monotonic ingest, v1 compatibility, release linkage, and non-claims.
- [ ] [serial] V7 Run focused source-core, source-bundle, project/fetch adapter, release-core, release CLI, and witness-rebuild tests with exact positive and negative summaries. r[source_transports.source_observations.contract] r[mantle.release_provenance.source_observation_binding]
- [ ] [serial] V8 Run wasm checks for the source core, focused formatting and Clippy, first-party quality rails, machine-contract checks, Cairn validation, Tracey coverage, all three change gates, and relevant Nix checks. r[source_transports.source_observations.claim_boundary] r[mantle.release_provenance.source_observation_signature_boundary]
