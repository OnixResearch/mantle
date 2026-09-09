# ADR 0059: Bind source observations without new signature authority

## Status

Proposed

## Context

Mantle source bundles identify canonical payloads with BLAKE3 and already support fixed URLs, local paths, VCS snapshots, mirrors, bootstrap archives, and proof inputs. Origin, revision, projection, and adapter facts can still remain in string metadata.

The [Atom Reforged architecture](https://nrd.sh/blog/atom-reforged.html) links source, revision, path, snapshot, and publication. Mantle needs the source relation, but it is a build tool rather than a package ownership or naming system.

Release attestations already sign the release-evidence manifest. Source review, build witnesses, and release signing have separate authority roles.

## Decision Drivers

- Bind immutable revision and snapshot facts to measured content.
- Keep URLs, mirrors, branches, tags, and local display paths non-authoritative.
- Preserve source-bundle v1 compatibility without inventing missing provenance.
- Make conflicting ingest fail without state mutation.
- Avoid duplicate signing and revocation policy.

## Decision

Mantle will define a versioned source observation with checked source kind, locator class, immutable revision when applicable, normalized projection, snapshot profile, content BLAKE3, and domain-separated observation BLAKE3.

A locator records where bytes were observed. It is not canonical content identity or ownership authority. Git identity uses the declared object format and immutable revision. A mutable ref can remain a non-authoritative hint. Mantle will not infer repository authority from one genesis commit.

A pure ingest planner returns add, identical reuse, identity conflict, or invalid rejection. Add uses atomic create-new publication. Reuse does not write. Every rejection preserves existing records, payloads, pins, roots, and readiness state.

Source-bundle v1 remains readable. A compatibility adapter creates a stronger source observation only when all required facts are present and unambiguous. Other valid v1 records keep a provenance-unavailable disposition.

Release evidence binds the accepted source-observation digest. The existing release signature covers that manifest linkage. Mantle will not add a source-observation signature role.

## Alternatives Considered

### Adopt Atom's package claim and publication protocol

Rejected because package naming, ownership, and version authority belong above Mantle's build-shaped boundary.

### Use URL or Git genesis as canonical source identity

Rejected because locations move, repositories can have multiple roots, and history can be rewritten.

### Backfill all v1 records from metadata

Rejected because incomplete metadata cannot support an honest immutable source relation.

### Sign each source observation separately

Rejected because the existing release, review, and witness roles already authenticate their own decisions.

## Consequences

- Source observation logic moves into a pure source core.
- Source adapters must name immutable revision and snapshot rules.
- New evidence gains one versioned identity while legacy bytes remain stable.
- Release verification gains an exact source-observation link without a new trust root.
- The observation proves a measured relation only. It does not prove ownership, review quality, or build correctness.
