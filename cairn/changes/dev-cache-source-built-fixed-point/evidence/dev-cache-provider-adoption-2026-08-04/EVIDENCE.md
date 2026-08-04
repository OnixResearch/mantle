# Dev cache provider-adoption increment evidence (2026-08-04)

Second increment for `dev-cache-source-built-fixed-point`. Builds on the
foundation increment. This records what changed and what validated.

## What is implemented and committed

Follow-on commits on `pi/dev-cache-source-built-fixed-point` (after the
foundation commit and a push).

- Provider cache now has a **writer**: after a successful cold dev run,
  `write_dev_provider_cache` publishes the StageX and native provider store
  subtrees plus a receipt-validated entry keyed by the current source-authority
  + policy digests.
- Provider cache now has a real **runtime adoption path**: on a validated hit,
  `run_attempt` skips the StageX transition, StageX provider publication, and
  native-provider build, copies the cached provider store subtrees into the
  fresh staging store (`adopt_cached_provider_subtrees`), and continues with a
  fresh host-tools + Rust provider + Cargo-free fixed point built against the
  adopted providers.
- Adopted runs are routed to a **dev-labeled outcome**: they record a
  dev-cache-hit transcript and a `dev-adopted-report.json`, and explicitly do
  NOT write the promoted deterministic receipt and do NOT publish to the
  `latest` / `latest-source-built-fixed-point` success aliases. A cached
  adoption can therefore never be mistaken for a fresh promoted proof.

## Validation evidence (same-turn)

Working env uses a gcc/mold linker override because the documented clang/mold
store paths were GC'd on this host.

- `cargo test -p mantle --bin mantle source_built_fixed_point` -> `46 passed;
  0 failed; 1 ignored` (includes the new adoption-copy test)
- First-party clippy reports no findings in
  `source_built_fixed_point_shell.rs` or `source_built_fixed_point_dev_cache.rs`.
  `-D warnings` still fails on three pre-existing violations in
  `full_source_rust_binding_shell.rs`, `realization_routing.rs`, and
  `stagex_transition.rs`.
- `rustfmt --check` on both touched leaf files -> clean; `git diff --check` ->
  clean.
- `cairn validate` and the proposal/design/tasks gates were already PASS and
  remain so.

## Not re-run in this increment

- The multi-hour full-source fixed-point proof and a full cold -> cached ->
  adopt round trip are not executed here; the adoption writer/lookup/copy logic
  is exercised at the unit level only.
- A fresh emitted promoted bundle is not reproduced.
- Cross-run multi-stage resume beyond the same-staging-dir transition resume is
  still partial (per-stage markers are persisted in the staging dir; the
  transition execution-tree snapshot needed for a fresh-dir resume is not yet
  stored in the cache).
- `bootstrap_eval` `gcc_native_diagnostic_uses_runtime_tcc_without_autotools_claims`
  still fails on this tree (asserts only on `bootstrap/gcc-4.0-native.ncl`
  content, untouched here; pre-existing).

## Remaining work for full completion

- Store a transition execution-tree snapshot so `--dev-resume` can restart from
  a fresh staging dir, not only the same interrupted dir.
- Confirm the adopted path with a real emitted run and a fresh promoted cold
  bundle.
