# ADR 0031: Hydrate fresh-clone inputs from source bundles

## Status

Accepted (2026-07-17)

## Context

Mantle's fixed-point self-build validates an explicit `vendor-deps/` directory against `Cargo.lock` and Cargo checksum manifests. That directory is generated, ignored by Git, and currently about 811 MiB. The legacy seed path also needs the pinned musl.cc provider payload in Mantle source state before `bootstrap --fetch --offline-source-preflight` can avoid a live fetch.

The successful fixed-point proof repaired and validated one prepared checkout. A fresh Git clone still lacks both payloads. Committing the generated vendor tree, trusting ambient Cargo/Nix caches, or adding an unrelated archive format would either burden normal repository operations or duplicate existing source-bundle identity and safety rules.

## Decision Drivers

- Reconstruct explicit inputs without ambient caches or network access.
- Require artifact authority independent of a bundle's self-declared digest.
- Reuse canonical path, symlink, mode, BLAKE3, provider-profile, import, and pin semantics.
- Validate Cargo lock/package/file checksums before making the vendor tree visible.
- Preserve atomic no-replace publication and deterministic rollback.
- Keep hydration evidence separate from fixed-point, compiler, seed-trust, and release claims.

## Decision

Mantle uses `mantle-source-bundle-v1` as the fresh-clone handoff. The `fresh-clone-inputs` bootstrap profile carries exactly the legacy provider archive, provider manifest, and vendored Cargo directory source; broader `self-build-proof` profiles remain accepted. The operator supplies the bundle plus an independently obtained expected manifest BLAKE3 to `mantle source bundle hydrate-self-build`.

The pure hydration planner validates the bundle and expected identity, then requires exactly one vendored-Cargo record, one legacy provider archive record, and one provider manifest record with consistent bootstrap profile metadata. The source-bundle per-file bound is raised from 16 MiB to a named 64 MiB limit because the pinned provider contains compiler executables up to about 26 MiB; larger files remain rejected before payload allocation. Bootstrap archives may preserve intentional case-distinct Linux kernel headers, but materialization is recanonicalized and identity-checked so a case-collapsing filesystem fails closed. Non-bootstrap source records retain the case-collision ban. The shell materializes the vendor record under a checkout-local staging root, validates it against the fresh clone's `Cargo.lock`, `.cargo/vendor-config.toml`, package checksums, and file checksums, and publishes `vendor-deps/` with Linux `renameat2(RENAME_NOREPLACE)`. It then imports and pins all verified records into the selected Mantle state directory. Persistence failure removes only the vendor directory created by that invocation before returning failure.

A successful machine report binds the source-bundle manifest BLAKE3, vendor content BLAKE3, provider archive content BLAKE3, import counts, and pin state. It contains no credential, cache, temporary, or checkout path identity.

## Alternatives Considered

### Commit `vendor-deps/`

Rejected because it adds roughly 811 MiB of generated source to every clone and creates broad churn on ordinary lock updates.

### Use only a Nix fixed-output closure

Rejected as the primary contract because it makes Nix closure transfer mandatory and duplicates Mantle source-record and pin authority. Nix can still transport the bundle as an opaque file.

### Introduce a tar-specific hydration format

Rejected because it would need a second implementation of path traversal, symlink, mode, payload digest, provider metadata, and no-clobber validation.

### Trust the bundle's internal digest

Rejected because modified payload and metadata can be paired with a recomputed self-declared digest. The expected BLAKE3 must arrive out of band or through a stronger signed publication channel.

## Consequences

- A fresh clone can reconstruct its ignored Cargo source and pinned legacy provider state from one identity-bound handoff.
- Existing `vendor-deps/` paths are never replaced; hydration is Linux-only until an equivalent atomic no-replace primitive is implemented elsewhere.
- The current JSON source bundle hex-encodes payload bytes and can be larger than a packed archive. Existing resumable transfer can carry it; a compact chunked encoding is future work.
- Source-bundle readiness proves only declared input availability and identity. Full fixed-point self-hosting, completeness for future undeclared bootstrap sources, compiler correctness, seed trust removal, release reproducibility, independent rebuild agreement, deployment, and full Cargo compatibility remain separate claims.
