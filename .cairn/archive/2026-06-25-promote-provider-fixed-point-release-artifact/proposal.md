## Why

Mantle can already bundle and verify a provider-backed Cargo-free fixed-point proof next to release evidence, but that proof is still only release-adjacent: verification reports the provider stage binary digest separately from the release artifact set. That leaves room to package a release binary from the legacy self-hosting proof while carrying a valid provider fixed-point proof for a different binary.

The next release-evidence step is to make provider fixed-point proof evidence bind to the packaged release artifact path before claiming the release artifact is provider-backed.

## What Changes

- Require `mantle release create --provider-fixed-point-proof <dir>` to fail unless the provider proof's fixed-point stage binary digest matches one of the bundled release binaries.
- Require `mantle release verify --require-provider-fixed-point-proof` to fail closed when the bundled or explicit provider proof validates but does not match the release binary artifact set.
- Report the matched release artifact path and digest alongside provider proof status when the binding succeeds.
- Keep deterministic-release, reproducibility, bootstrap, compiler-correctness, and full-Cargo-compatibility claims separate.

## Impact

- **Files**: `crates/crunch-release-core/src/manifest.rs`, `crates/crunch-release-core/src/lib.rs`, `src/cargo_free_self_build.rs`, `src/release_cmd.rs`, `src/release_evidence.rs`, release docs, Cairn specs/evidence.
- **Testing**: focused release core tests, provider fixed-point release create/verify tests, negative mismatch tests, `cargo test -p mantle --bin mantle provider_fixed_point`, `cargo test -p crunch-release-core`, Cairn validation/gates, and diff checks.
