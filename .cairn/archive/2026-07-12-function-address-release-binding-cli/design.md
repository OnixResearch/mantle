## Design

The existing release evidence core already validates function-address release evidence inside a manifest. The CLI adds an operator-facing shell that reads Valence sidecar/receipt metadata and optional Kamacite receipt metadata, binds them to a release artifact set, and writes a deterministic binding receipt.

### Decisions

1. **Core stays release-local.** Mantle validates paths, digests, roles, schemas, claim scopes, source archive identity, binary identity, and non-claims only.
2. **CLI owns I/O.** File reads, path resolution, and receipt writes stay outside `crunch-release-core`.
3. **Optional remains default.** The binding command supports optional and required modes so adoption stays gradual.
4. **Public identities only.** The shell deserializes only `schema_version`, logical `receipt_hash`, and the optional Valence-to-Kamacite `kamacite_receipt_hash`; Mantle never parses function records or infers upstream semantics.
5. **Reopened bytes stay bound.** After bundle verification, the shell re-hashes each no-follow receipt read against the manifest's artifact-byte digest so replacement races fail before output.
6. **Output never clobbers.** Canonical bytes are synchronized through a same-directory temporary file and committed with no-clobber persistence. A deterministic policy `FAIL` receipt may be preserved while the command exits non-zero.

### Validation shape

Positive fixtures cover required mode with Valence sidecar, Valence receipt, and Kamacite receipt; optional mode without Kamacite; distinct logical and artifact identity domains; and binary/source archive linkage. Negative fixtures cover missing sidecar, stale artifact bytes, stale logical links, wrong role/schema, unsupported claim scope, overclaiming non-claim text, and output collisions.
