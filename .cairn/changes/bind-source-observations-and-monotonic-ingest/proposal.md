# Change: Bind source observations and monotonic ingest

## Why

Mantle source bundles already bind canonical payload bytes with BLAKE3 and support VCS snapshots. The durable `SourceRecord` still keeps most origin and adapter facts in string maps. Release `SourceAcquisition` also uses string fields for kind, URL, revision, reference, and profile.

The accepted specifications require idempotent import and VCS revision checks. They do not yet define one typed source observation that links locator class, immutable revision, normalized projection, snapshot profile, and resulting content identity. They also do not state the full no-mutation law for conflicting or invalid ingest.

Mantle needs those relations without becoming a package registry, treating a URL as authority, or adding another signature system.

## What Changes

- Add a versioned, backend-neutral source observation with checked source kind, locator class, immutable revision when applicable, normalized projection, snapshot profile, and BLAKE3 content identity.
- Give source observations a domain-separated canonical BLAKE3 identity.
- Add a pure ingest planner with add, identical reuse, conflict rejection, and invalid rejection outcomes.
- Make successful source ingest monotonic and make every rejected ingest preserve durable state.
- Extend the pinned durable-file-publication adoption from remote-attempt objects to immutable source records, observation sidecars, and pins.
- Preserve source-bundle v1 reading and accepted existing identities through explicit compatibility projection.
- Bind accepted source-observation identity into release evidence without adding a separate source signature.
- Keep package ownership, package names, versions, and publisher authority outside Mantle.

## Dependencies

- `extend-nominal-types-to-trust-boundaries` owns checked fetch URLs, Git revisions, paths, and digest roles.
- `bind-source-review-evidence-to-releases` owns external reviewer authority and must use the same exact release-source subject.
- `prove-source-built-mantle-fixed-point` currently exercises source-bundle hydration and must stabilize before a source-bundle schema migration lands.
- Valence owns cross-project evidence identity and linkage semantics.

## Non-Goals

- A Mantle package registry, namespace, ownership claim, or version resolver.
- Treating a mutable Git ref, tag, remote URL, or one genesis commit as repository authority.
- Adding a new signature suite or letting source observations satisfy review, witness, or release-signature policy.
- Requiring one ecosystem's source fields for unrelated adapters.

## Impact

- **Affected specs:** `source-transports`, `release-provenance`, `durable-file-publication-adoption`
- **Affected code:** a new pure source core, source bundle adapters, release evidence projection, and focused project/fetch adapters
- **Compatibility:** v1 source bundles remain readable; new observations use an explicit versioned contract
- **Testing:** canonical identity, malformed input, monotonic ingest, atomic failure, compatibility, release linkage, and Cairn gates
