# Design: Bind source observations and monotonic ingest

## Context

`SourceRecord` has a typed kind and BLAKE3 content digest, but locator, revision, and adapter-specific facts can remain in untyped metadata. `SourceAcquisition` has explicit fields, but its kind and identity-bearing values remain structural strings.

A source location answers where bytes were observed. It does not prove ownership or trust. Content identity, immutable revision, and release policy are separate facts.

## Decisions

### Decision: Add a pure source-observation core

**Choice:** Add a no-std-capable core over owned `alloc` values. The core will admit a structural wire value into a checked `SourceObservation` containing:

- schema and source-kind versions;
- a locator class with bounded, redacted structural data;
- Git object format and immutable revision when the source is Git;
- a normalized relative projection path;
- snapshot profile name and version;
- canonical payload BLAKE3;
- a domain-separated observation BLAKE3.

Filesystem reads, URL access, Git object loading, archive construction, and release bundle I/O remain in std shells.

**Rationale:** The relation is deterministic logic and can be tested without network, Git, or filesystem setup.

### Decision: Treat locators and mutable refs as observations

**Choice:** A URL, mirror, Git remote, branch, tag, or local display path will not be canonical source identity. Git observations use the immutable object ID, object format, projection, snapshot profile, and measured BLAKE3. Mutable refs can be recorded only as hints.

Secret-bearing URL userinfo and unapproved query fields will be rejected or redacted before canonical evidence is produced.

**Rationale:** Locations can move, mirrors can differ, and refs can advance. The measured content relation must remain stable.

### Decision: Use a pure monotonic ingest plan

**Choice:** The core planner returns one of four outcomes:

- `Add` when the semantic identity is absent;
- `ReuseIdentical` when canonical existing and incoming records match;
- `RejectIdentityConflict` when one identity names different canonical content or provenance;
- `RejectInvalid` when admission fails.

Only `Add` authorizes a durable create-new commit through the pinned `durable-file-publication` component. Reuse performs no write. Rejection leaves records, payloads, pins, roots, and readiness state unchanged.

The shell preserves the component's commit distinctions. It does not erase a visible destination after `CommittedDurabilityUnknown`. That result remains different from rejection and durable success.

**Rationale:** This makes idempotence, conflict handling, and no-loss behavior one reviewable rule.

### Decision: Keep v1 compatibility explicit

**Choice:** Existing source-bundle v1 records remain readable. A compatibility adapter can derive a partial observation only when v1 metadata contains sufficient unambiguous facts. Otherwise the record remains a valid legacy content record with an explicit `provenance-unavailable` disposition.

New source observations use a versioned field or bundle version. They do not silently reinterpret old metadata.

**Rationale:** Backfilling provenance from incomplete string maps would create false evidence.

### Decision: Reuse existing release signatures

**Choice:** Release evidence binds the source-observation digest and exact source content digest. The existing release attestation signs the enclosing release-evidence manifest. Reviewed-source policy binds the same source subject through its external Cairn and Valence evidence.

Mantle will not add a second source-signature role.

**Rationale:** Another signature would duplicate authority and complicate revocation without adding a new verified fact.

## Functional core and imperative shell

The core owns structural admission, canonicalization, observation identity, ingest planning, and compatibility decisions. The shell owns acquisition, measurement, durable component adaptation, rollback before commit, pin persistence, and report rendering.

## Risks and trade-offs

- A new versioned record increases compatibility work. Golden fixtures must cover v1 and the new form.
- Locator redaction can remove useful diagnostics. Reports can keep separate non-canonical display hints.
- VCS adapters differ in object semantics. Each adapter must name its immutable revision grammar and snapshot profile.
- Source-review work can duplicate source identities. Both changes must share one source-subject digest.

## Claim boundary

A valid source observation proves only the recorded relation between supplied locator facts, immutable revision facts, projection rules, snapshot profile, and measured content. It does not prove source ownership, upstream intent, review quality, license compliance, build correctness, or release eligibility.
