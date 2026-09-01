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

The enforced checker reports zero findings, 472 evidence-backed
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
- **Next action:** commit the synchronized archive, push the verified branch,
  and integrate it into `origin/main`.

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

## Later full-check blocker: GC and overlay Tiger Style

The strict Tiger gate then found assertion-density, quantity-name, predicate,
bounded-growth, and ambiguous-parameter issues in `crunch-gc-core` and
`crunch-overlay-core`. The repair adds post-validation invariants, one explicit
rank-map bound, unit/predicate names, and a byte-increment newtype. It adds no
lint allowance. Both pre-change baselines passed. After the repair, all 23 GC
core tests, all eight overlay core tests, and focused Clippy pass. The Tiger
gate reports no remaining finding in either crate and advances to later cores.

## Strict gate progression

Later focused baselines also passed for composition, evaluator budgets, release,
portable clients, Mantlepkgs versions, build contracts, and filesystem NAR
observation. Their repairs use named request records, checked arithmetic,
post-validation invariants, bounded collections, fixed-width public frame
lengths, and smaller functions. No lint allowance was added. Post-change tests,
focused Clippy, root-package Clippy, and formatting pass.

The strict gate now reaches `crunch-store`. The preserved run contains 139
location-backed findings across 13 files:

- `provenance.rs`: 30;
- `roots.rs`: 26;
- `nario.rs`: 21;
- `overlay.rs` and `gc.rs`: 15 each;
- `pull.rs`: 12;
- `http_closure.rs`: eight;
- `composition.rs`: six;
- six remaining findings across `retention.rs`, `query.rs`, `layer.rs`,
  `handle.rs`, and `capability.rs`.

This is the next repository-wide blocker. It is not hidden, baselined, allowed,
or downgraded. The accepted parity claim remains independent from this source
quality debt.

## Final command outcomes

The following committed-source checks pass:

- blocker inventory enforcement and self-tests;
- proof-file tamper and missing-`b3sum` negative cases;
- focused tests for nine repaired core packages;
- focused and root-package Clippy with `-D warnings` and no new allowance;
- root and repaired-package formatting;
- bootstrap inventory, SpaceWasm rebuild, wasm-component/Octet rebuild, and
  durable-publication Nix checks;
- `nix flake check --no-build -L`;
- Cairn validation, Tracey 155/155, and proposal/design/tasks gates.

Both local-builder and ordinary full flake checks stop at the same
`checks.x86_64-linux.tigerstyle` store debt. Neither run reports the earlier
SpaceWasm or Octet fixed-output mismatch.

## Non-claims

This change does not claim that full `nix flake check -L` passes. It does not
claim `crunch-store` Tiger conformance, compiler correctness, seed correctness,
release reproducibility, deployment success, or wider parity than V98 already
proved.
