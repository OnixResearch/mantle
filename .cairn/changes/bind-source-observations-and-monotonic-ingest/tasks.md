# Tasks: Bind source observations and monotonic ingest

## Phase 1: Dependencies and baseline

- [ ] [serial] V1 Record current source-bundle v1 bytes, manifest BLAKE3 values, imported-state behavior, `SourceAcquisition` bytes, release identities, and focused test output before changes. r[source_transports.source_observations.compatibility] r[mantle.release_provenance.source_observation_binding]
- [ ] [serial] I1 Review and accept ADR 0059 for typed source observations, locator non-authority, monotonic ingest, v1 compatibility, and reuse of existing signature roles. r[source_transports.source_observations.contract] r[mantle.release_provenance.source_observation_signature_boundary]
- [ ] [depends:extend-nominal-types-to-trust-boundaries] I2 Reuse checked URL, Git revision, projection-path, profile, and BLAKE3 values instead of defining competing primitive wrappers. r[source_transports.source_observations.contract]
- [ ] [depends:bind-source-review-evidence-to-releases] I3 Align source observation and reviewed-source attachment on one exact release-source subject and keep reviewer authority external. r[mantle.release_provenance.source_observation_binding]
- [ ] [depends:prove-source-built-mantle-fixed-point] I4 Record the stabilized source-bundle and hydration compatibility boundary before changing source records. r[source_transports.source_observations.compatibility]

## Phase 2: Pure source core

- [ ] [serial] I5 Add a no-std-capable source core with structural wire DTOs, admitted source kinds, locator classes, immutable revisions, normalized projections, snapshot profiles, content BLAKE3, and domain-separated observation BLAKE3. r[source_transports.source_observations.contract]
- [ ] [serial] I6 Add pure locator-boundary checks that reject or redact secret-bearing location data and keep mutable refs outside canonical identity. r[source_transports.source_observations.locator_boundary]
- [ ] [serial] I7 Add the pure add, identical-reuse, identity-conflict, and invalid-rejection ingest planner with explicit state-preservation results. r[source_transports.monotonic_ingest]
- [ ] [serial] I8 Add explicit v1 compatibility projection and provenance-unavailable outcomes without changing legacy canonical bytes. r[source_transports.source_observations.compatibility]

## Phase 3: Source and release shells

- [ ] [serial] I9 Adapt fixed URL, Git, local logical source, package mirror, and opaque adapter inputs into admitted source observations. r[source_transports.source_observations.contract] r[source_transports.source_observations.locator_boundary]
- [ ] [serial] I10 Apply add plans through staged create-new publication, make identical reuse write-free, and preserve all durable state on rejection or interruption. r[source_transports.monotonic_ingest]
- [ ] [serial] I11 Bind source observation identity into release evidence and the existing release-attestation signature without adding a new signer role. r[mantle.release_provenance.source_observation_binding] r[mantle.release_provenance.source_observation_signature_boundary]
- [ ] [serial] I12 Add versioned machine-contract and Nickel review-contract updates for the new source observation and release binding. r[source_transports.source_observations.contract] r[mantle.release_provenance.source_observation_binding]

## Phase 4: Positive and negative verification

- [ ] [parallel] V2 Add positive canonicalization, rematerialization, mirror-equivalence, add, identical-reuse, legacy compatibility, and release-binding fixtures. r[source_transports.source_observations.contract] r[source_transports.monotonic_ingest] r[mantle.release_provenance.source_observation_binding]
- [ ] [parallel] V3 Add negative fixtures for mutable-ref drift, wrong revision, unsafe projection, unsupported profile, secret-bearing locator, malformed digest, contradictory fields, and incomplete legacy provenance. r[source_transports.source_observations.contract] r[source_transports.source_observations.locator_boundary] r[source_transports.source_observations.compatibility]
- [ ] [parallel] V4 Add mutation and interruption tests proving rejected or interrupted ingest leaves records, payloads, pins, roots, readiness, and release evidence unchanged. r[source_transports.monotonic_ingest]
- [ ] [parallel] V5 Add negative release fixtures for stale observation identity, stale source bytes, wrong profile, unknown source signature, and cross-role signature substitution. r[mantle.release_provenance.source_observation_binding] r[mantle.release_provenance.source_observation_signature_boundary]
- [ ] [parallel] V6 Add golden v1 and new-version wire, canonical-byte, manifest, observation, and release-identity fixtures. r[source_transports.source_observations.compatibility]

## Phase 5: Documentation and lifecycle

- [ ] [serial] I13 Document source observation fields, adapter rules, monotonic ingest, v1 compatibility, release linkage, and all non-claims. r[source_transports.source_observations.claim_boundary]
- [ ] [serial] V7 Run focused source-core, source-bundle, project/fetch adapter, release-core, release CLI, and witness-rebuild tests with exact positive and negative summaries. r[source_transports.source_observations.contract] r[mantle.release_provenance.source_observation_binding]
- [ ] [serial] V8 Run wasm checks for the source core, focused formatting and Clippy, first-party quality rails, machine-contract checks, Cairn validation, Tracey coverage, all three change gates, and relevant Nix checks. r[source_transports.source_observations.claim_boundary] r[mantle.release_provenance.source_observation_signature_boundary]
