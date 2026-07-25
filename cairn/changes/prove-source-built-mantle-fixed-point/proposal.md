## Why

Mantle has separate fixed-point evidence for the admitted full-source C provider and for a source-built Rust/native closure, but no proof constructs and consumes both within one authenticated lineage. The checked real-self-build parity evidence is also stale: it embeds `mantle-deterministic-proof-receipt-v1`, while the current verifier requires v2 content-bound rebuild authority.

A full-bootstrap fixed-point claim requires a fresh proof from source inputs, not composition of old output receipts. The proof must construct the native and Rust providers, build stage1 Mantle, use stage1 Mantle to build stage2 through the same Cargo-free path, and retain strict zero-fetch/zero-fallback evidence with matching BLAKE3 binaries.

## What Changes

- Add a full-bootstrap proof mode that consumes authenticated source bundles and constructs the StageX native provider plus full-source-bound Rust provider before Mantle compilation.
- Execute both Mantle stages through the receipt-bound Cargo-free topology and strict hermeticity policy.
- Emit current `mantle-deterministic-proof-receipt-v2` evidence linking source, lineage, toolchain closure, stage authority, outputs, protected execution, and non-claims.
- Preserve all failed and mismatched attempts without updating successful-proof aliases.

## Dependencies

- `bind-full-source-rust-provider`.
- `materialize-stagex-lineage-provider` (which transitively depends on both native parity changes).

## Impact

- **Files**: self-build/Cargo-free orchestration, source-bundle profiles, deterministic proof/release evidence, parity inputs, proof scripts/tests, documentation, and lifecycle evidence.
- **Testing**: pure proof-plan tests; negative authority/fallback/tamper cases; real multi-stage proof; v2 receipt verification; release evidence verification; Cairn gates.