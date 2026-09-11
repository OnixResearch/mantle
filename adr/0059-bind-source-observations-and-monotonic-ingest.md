# ADR 0059: Source observations bind immutable facts; ingest is monotonic

## Status

Accepted (in the `bind-source-observations-and-monotonic-ingest` change).

## Context

Source-bundle v1 records keep origin and adapter facts in string maps, and
release `SourceAcquisition` uses structural strings for kind, URL, revision,
reference, and profile. A locator answers where bytes were observed; it does
not prove ownership, trust, or content identity, and mirrors and mutable refs
can move.

## Decision

Add a pure `no_std + alloc` core (`crates/crunch-source-core`) that admits a
structural request into a checked `SourceObservation` binding: source kind,
locator class, immutable Git revision and object format when applicable,
normalized relative projection, snapshot profile name and version, payload
BLAKE3, and a domain-separated observation BLAKE3.

- Locators are observations. Userinfo, fragments, and secret-bearing query
  fields are rejected; other query fields require explicit policy approval.
  Mutable refs may be recorded only as hints and never enter the identity.
- Git SHA-1 object IDs are admitted as protocol interoperability, not as
  stack identity; SHA-256 revisions are admitted at equal rank.
- Ingest planning is pure and returns exactly one of `Add`,
  `ReuseIdentical`, `RejectIdentityConflict`, or `RejectInvalid`. Only `Add`
  authorizes a durable create-new commit; reuse and rejection preserve
  durable state.
- Legacy v1 records remain readable. A projection is derived only from
  unambiguous metadata; otherwise the record keeps a valid legacy content
  identity with an explicit `provenance-unavailable` disposition. Legacy
  canonical bytes are never rewritten.

## Consequences

- Adapters (fixed URL, Git, local logical, package mirror, opaque) map onto
  one admitted observation shape; filesystem, network, Git, and release I/O
  stay in std shells.
- Release evidence binds the observation digest alongside the exact source
  content digest; no second signature role is introduced.
- Backfilled provenance is impossible by construction: incomplete legacy
  metadata yields an explicit unavailable disposition instead.
