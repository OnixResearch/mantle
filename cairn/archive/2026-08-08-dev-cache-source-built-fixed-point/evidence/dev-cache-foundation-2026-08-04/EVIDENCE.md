# Dev cache foundation evidence (2026-08-04)

Scope: first implementation increment for `dev-cache-source-built-fixed-point`.
This records what is implemented, what validated, and what remains, so no claim
outruns current evidence.

## What is implemented and committed

Commit `d1509792` on `pi/dev-cache-source-built-fixed-point`.

- New pure core `src/source_built_fixed_point_dev_cache.rs`:
  - dev provider-cache key from source-authority + five policy digests
  - provider-cache entry receipt re-validation (exact schema/plan/key/policy and
    native-provider digest match; any mismatch is a hard miss)
  - stage-marker validation against the current plan, stage id, and fresh digest
  - fast-fail decision (report exact prior source digest or proceed)
- Shell wiring in `src/source_built_fixed_point_shell.rs`:
  - `--dev-fast-fail` short-circuits before a full run when the current source
    profile matches the last published fixed-point receipt binding
  - `--dev-provider-cache` enables content-addressed store snapshot write after a
    completed dev run and store seeding on the next same-plan run
  - per-stage completion markers persisted and a dev resume guard that skips a
    trusted StageX transition when its artifacts are already present
  - `dev_cache_adoption` decision + dev-cache-hit transcript record
- CLI flags in `src/main.rs` (`--dev-provider-cache`, `--dev-resume`,
  `--dev-fast-fail`), all `requires = "source_built_fixed_point"` and off by
  default, so the promoted cold path is not altered.

## Validation evidence

Command results captured same-turn (working env uses a gcc/mold linker override
because the documented clang/mold store paths were GC'd on this host):

- `cargo test -p mantle --bin mantle source_built_fixed_point` -> `45 passed;
  0 failed; 1 ignored` (includes dev-cache core and shell helper tests)
- `cargo test -p mantle --bin mantle self_build_cli` -> `12 passed; 0 failed`
  (includes `self_build_cli_accepts_dev_cache_flags_without_promoting_cold_path`)
- `rustfmt --edition 2021 --check` on both touched leaf files -> clean
- `git diff --check` -> clean
- First-party clippy on the `mantle` bin reports no findings in
  `source_built_fixed_point_dev_cache.rs` or `source_built_fixed_point_shell.rs`.
  `-D warnings` still fails on three pre-existing violations in
  `full_source_rust_binding_shell.rs`, `realization_routing.rs`, and
  `stagex_transition.rs` that are unrelated to this change.
- `cairn validate --root .` (cairn built from `OnixResearch/cairn` source) ->
  `valid: true`
- `cairn gate proposal|design|tasks dev-cache-source-built-fixed-point` ->
  `verdict: PASS` for all three

## Not re-run in this increment

- The multi-hour full-source fixed-point proof is not run; the runtime
  provider-reconstruction skip and cross-run multi-stage resume are exercised at
  the decision/helper level, not end to end.
- The promoted cold path is code-gated and unchanged, but a fresh emitted
  promoted bundle was not reproduced here; confirm before release.
- `bootstrap_eval` integration `gcc_native_diagnostic_uses_runtime_tcc_without_autotools_claims`
  fails on this tree; it asserts only on `bootstrap/gcc-4.0-native.ncl` content,
  which this change does not touch, and is treated as pre-existing.

## Remaining work for full completion

- Wire the true provider-adoption skip that stores the StageX transition
  execution tree and reconstitutes the downstream provider handles so a
  validated dev hit is not reconstructed.
- Exercise multi-stage resume across a fresh staging dir (not only same-dir
  transition resume).
- Confirm the promoted cold path with a fresh emitted bundle.
