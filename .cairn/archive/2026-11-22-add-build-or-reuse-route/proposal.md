# Change: Add a build-or-reuse route keyed by Mantle's builder signature

## Why

The phase-1 definition of done requires that an unchanged foreign job is reused
from the store without re-running its builder, while a changed job still builds.
Today `cache-inputs-build-roots-v1` refuses every selected root that is already
present, because a present root could only be attested by a substituted cache
key. That refusal is safe but makes a repeated realization impossible.

The adopted decision (`step2-decision.md`, option A) narrows reuse to roots that
Mantle itself built earlier and signed with its builder key. Cache-key
signatures neither qualify nor disqualify a root, and reuse never means fresh
work.

## What Changes

- Add the preserved-path route `cache-inputs-build-roots-v2`
  (`CACHE_INPUTS_BUILD_OR_REUSE_ROUTE`), selected with
  `plan --realization-route cache-inputs-build-roots-v2`, legal only with
  `preserve-cache-paths-v1` and with the same structural invariants as v1
  (fixed-output selected roots still refused).
- Add the receipt schema `mantle-foreign-realization-receipt-v2` with a required
  `roots` array carrying per-root disposition (`built` or `reused`), signer
  fingerprint and role, trust-policy digest, and the build or reuse link. Every
  other route keeps emitting receipt v1 unchanged.
- Add builder-key identity: a fingerprint function and trust-policy digest, a
  verify-only repeatable `realize --trusted-builder-public-key
  <nix-name:base64>`, and a per-root exclusive lock under the realize state
  directory.
- Realize on v2 classifies present roots with a pure `classify_present_root`
  against a write-once built-root index; reusable roots perform no hydration and
  no build, absent roots follow the v1 path, and a typed refusal blocks reuse of
  a present root that cannot be attested.
- Record the operator-facing reuse contract in the operator guide and the
  trust-model documentation.

## Impact

- **Files**: `src/foreign_executable_plan.rs`, `src/foreign_realization.rs`,
  `src/foreign_realization_shell.rs`, `src/foreign_realization_receipt.rs`,
  `src/foreign_import_cmd.rs`, `src/signing_key.rs` (audit only),
  `crates/crunch-build/src/signing.rs`, `crates/crunch-store/src/handle.rs`,
  `vendor/nix-compat/src/narinfo/verifying_keys.rs`,
  `tests/cache_inputs_build_route.rs`, `docs/foreign-import-operator-guide.md`,
  `docs/foreign-import-trust-model.md`.
- **Testing**: focused `cargo test -p mantle` suites (`--test
  cache_inputs_build_route`, `--test foreign_import_cli`, `--bin mantle`
  modules), focused `-p crunch-build`/`-p crunch-store` lib tests, `cargo fmt
  --check` and strict Clippy for the touched crates, Cairn validate and the
  three change gates.
- **Non-goals**: Kiln adapter support for receipt v2, verify force-rebuild,
  reuse from a cache (`source: cache`), and key rotation by rebuild-compare-co-sign.
