## Design

The existing release evidence core already validates function-address release evidence inside a manifest. The CLI adds an operator-facing shell that reads Valence sidecar/receipt metadata and optional Kamacite receipt metadata, binds them to a release artifact set, and writes a deterministic binding receipt.

### Decisions

1. **Core stays release-local.** Mantle validates paths, digests, roles, schemas, claim scopes, source archive identity, binary identity, and non-claims only.
2. **CLI owns I/O.** File reads, path resolution, and receipt writes stay outside `crunch-release-core`.
3. **Optional remains default.** The binding command supports optional and required modes so adoption stays gradual.
4. **No function parsing.** Mantle treats Valence and Kamacite artifacts as opaque external evidence.

### Validation shape

Positive fixtures cover required mode with Valence sidecar, Valence receipt, and Kamacite receipt; optional mode without Kamacite; and binary/source archive linkage. Negative fixtures cover missing sidecar, stale digest, wrong role/schema, unsupported claim scope, and overclaiming non-claim text.
