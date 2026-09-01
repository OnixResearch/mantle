# Full-check blocker validation

## Baseline

The report-only inventory found 115 actionable findings across three classes:
72 `bridge-output`, 40 `compiler-runtime-crash-boundary`, and three
`placeholder-deferred`. It recorded 355 evidence-backed suppressions and no
promotion claims.

The findings separate into exact V98 source or receipt markers and structural
false positives. Structural examples include a false bridge-use field, a named
timeout budget, and a positive static-link smoke stage.

## Current focused result

The enforced checker reports zero findings, 470 evidence-backed
classifications, and zero promotion claims. A temporary byte append to
`bootstrap/stagex-transition-lineage.ncl` made enforcement fail. The test then
restored the original BLAKE3 identity.

## Review checkpoint

- **Question:** Can the inventory become clean without weakening its unknown or
  changed-marker behavior?
- **Inspected evidence:** V98 independent verification, exact source file
  identities, the 115-finding report, structural near-miss tests, and a real
  proof-file tamper run.
- **Decision:** accept exact proof-bound and explicit structural classification;
  keep all other marker text actionable.
- **Owner:** `close-full-check-blockers`.
- **Next action:** run full local-builder and ordinary Nix checks.

## Fixed-output and transport results

The original SpaceWasm lane failed while `importCargoLock` requested
`wasm-encoder`, `wasmparser`, and `wasm-smith` through the blocked
`crates.io/api` endpoint. Component-scoped Crane vendoring reached
`static.crates.io` and produced the complete bundle. A second `--rebuild` check
returned the same output path:

```text
/nix/store/s10dpbgap6vk219sgvfqlmnp8kj3x9rf-mantle-spacewasm-reference-bundle-e24cf09355a90497148eb5029fdb8e3400bd63e3
```

The wasm-component toolchain also passed `--rebuild`. Its output includes the
pinned Octet package and profile checks. No source revision, lockfile, Rust
version, target, package command, or feature selection changed.

## Later full-check blocker: durable publication freshness

The next local full-check failure was
`checks.x86_64-linux.durable-file-publication-adoption`. Its accepted Nickel
receipt still named older `Cargo.toml`, `Cargo.lock`, `flake.nix`, and
`flake.lock` BLAKE3 identities. The Nickel source, generated JSON, receipt
BLAKE3, and pure validator now bind the current files. Positive and negative
Nickel tests pass, and the focused Nix check passes.
