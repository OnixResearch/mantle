# Change: Let `foreign-import realize` choose the build core count

## Why

Mantle's sandbox environment fixes `NIX_BUILD_CORES=1`, and the Octet pilot job pins its CPU
affinity to that value. On one core its Lean stage exceeds Octet's 45-minute rail budget; Nix
builds the same derivation in 36 minutes with four cores. The landed route
`cache-inputs-build-roots-v2` therefore cannot reach a built root for that job on this host.

Evidence: `/home/brittonr/wt/convergence-evidence/build-or-reuse/pilot-v2/README.md` (root cause
section).

## What Changes

- Add `foreign-import realize --build-cores <N>`, integer `1..=64`, default `1`. A value above the
  default is written as `NIX_BUILD_CORES` for every build the realization runs; at the default the
  build environment is exactly today's (sandbox profiles export `1`, Nix-protocol profiles leave it
  unset).
- Refuse a non-default value on any route other than `cache-inputs-build-roots-v2`, before any
  store mutation, because receipt v1 has no field to record it.
- Record an explicit value in receipt v2 as `roots[].built.build_cores` (optional, omitted at the
  default) so default receipts are unchanged and receipts written by `6594e7fd1` keep parsing and
  keep their digests. Reuse ignores the value: a root built with any core count is reusable.
- Thread the value through the pipeline as a `BuildConfig` field (default `1` in every
  constructor) into the sandbox environment, where a derivation-declared value still wins, exactly
  as before.
- Regenerate the operator command contract and document the flag.

## Impact

- **Files**: `src/foreign_import_cmd.rs`, `src/foreign_realization_shell.rs`,
  `src/foreign_realization_receipt.rs`, `crates/crunch-pipeline/src/lib.rs`,
  `crates/crunch-build/src/build_request.rs`, `crates/crunch-build/src/orchestrate.rs`,
  `tests/cache_inputs_build_route.rs`, `tests/fixtures/foreign-import/realize-two-node-builder.sh`,
  `config/operator-command-descriptors.json`, `config/operator-surfaces.ncl`,
  `config/operator-surfaces.json`, `config/operator-command-catalog.json`,
  `docs/generated/operator-command-reference.md`,
  `docs/generated/canonical-operator-workflow.md`,
  `docs/foreign-realization-operator-guide.md`.
- **Testing**: `-p mantle` bin unit modules (`foreign_`, `realize_effect_plan`), focused
  `-p crunch-build` lib tests, `tests/cache_inputs_build_route.rs`, `tests/foreign_import_cli.rs`,
  `tests/operator_diagnostics.rs`, `cargo fmt --check -p mantle`, strict Clippy for `-p mantle`.
- **Non-goals**: deriving the core count from `--jobs`, changing the Octet rail budget, exposing
  the value to other routes, or recording it in receipt v1.
