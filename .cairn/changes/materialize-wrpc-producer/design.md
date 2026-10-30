# Design

The pinned upstream wRPC v0.15.0 commit is
`aa01409ed38bf7a978570ff567c4475376925df6`. Its exact unversioned hello WIT remains
unchanged; minimal source wrappers add callable run exports. The open client
imports hello, the server exports hello. Wasmtime 33.0.0/runtime0.29.0/
transport0.28.4 are the remote runtime cohort, distinct from the producer tool
manifest's pinned CLI cohort. No equivalence is claimed.

Nix imports generated Cargo locks and explicit vendored sources. The actual
`mantle wasm-component build` exporter executes both component pipelines. Each
produces a real materialization-bundle-v1, report, attestation and release binding.
An optional bounded runtime-only WAC request binds the exact final client once
and an identity-checked server, then runs the closed comparison. It cannot replace
the final portable client's RPC import. Its source/receipt/artifact are retained.

The outer manifest indexes all members and both bundle/profile identities. A
capability directory mirrors exporter logical paths without rewriting identities
or granting ambient filesystem-root access. Consumer source-tree snapshot/copy
support is an existing-protocol prerequisite owned by the parent verifier slice.

Generated profiles implement the consumer's exact reference encoding, not a
JSON-digest approximation. The selected world WIT content reference is bound.
All runtime WASI subsystems are denied; the declared non-WASI hello import remains.

No retries, local guest-build fallback, ordinary transport fixtures, fabricated
receipt decisions, hard-coded output digests, or unpublished immutable addresses
are admitted. The parent publishes the exact revision, materializes the package,
runs positive/negative real-byte cases, and observes real fabric RPC separately.
