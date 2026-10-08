# Bootstrap source pin re-verification on published main — 2026-10-08 (UTC)

All runs: Leviathan, isolated worktree `~/mantle-cairn-finish/wt/add-bootstrap-source-pins`
(branch `finish/add-bootstrap-source-pins`, created with
`git worktree add -b finish/add-bootstrap-source-pins … main` from published main
`e24bbbc2f803f59872e2e59370bd8ce0829c918e`), per-worktree `CARGO_TARGET_DIR`/`TMPDIR`,
`nix develop` dev shell (rustc 1.96.0-nightly, nickel 1.17.0), every job under
`mcf-guard.sh`. The BEFORE leg ran in a second detached worktree of the same main
(`wt/add-bootstrap-source-pins-baseline`). The 2026-10-01 receipts in this directory
are historical: they were produced on an integration checkout 89 commits behind
this main and are not relied on below.

## Extraction onto main

Source: `refs/heads/local/rescue/main-checkout-20261007` (`6ad3b30f`, base
`da00f584`). Only this change's paths were taken:

- Whole files (`git diff da00f584 6ad3b30f -- <paths> | git apply --3way`): this
  change directory (design, tasks, spec delta, 2026-09-30/10-01 evidence), ADR 0086,
  `bootstrap/pins/{cmake,picolibc}.toml`, `bootstrap/pins/generated/{cmake,picolibc}.json`,
  `bootstrap/pins/generate_readers.py`, the three migrated recipes,
  `crates/crunch-project-core/src/bootstrap_pins.rs` (+ `pub mod bootstrap_pins;`),
  `src/bootstrap_pin_cmd.rs`, `docs/bootstrap-stage0-inventory.md`, and deletion of
  `scripts/check-bootstrap-source-pins.rs`. None of these paths changed between
  `da00f584` and main, so every hunk applied cleanly.
- Not taken (other owners, found in the same rescue diff): `bootstrap/pins/cargo-shared-lock-hashes-v1`
  (lock-vendor), the `nix-gateway` profile/test in `mantle-portable-client-core`,
  the AGENTS.md gate rewrite, the `build_run_context` command-family hunk, `remote live`
  policy rows, other ADR rows.
- Hunk-extracted shared files: `src/main.rs` (module, `BootstrapPin` command,
  `BootstrapPinAction`, root/label/dispatch arms — 47 added lines),
  `Cargo.toml` (`crunch-project-core` root dependency), `Cargo.lock` (mantle
  dependency entry), `crates/mantle-portable-client-core/src/lib.rs`
  (`PIN_IO`, `bootstrap-pin` profile, `ROOT_COMMAND_COUNT` 39 → 40),
  `config/operator-surfaces.ncl` (three `bootstrap-pin` command rows and one
  platform-profile row), `adr/README.md` (0086 row).
- Regenerated only through the documented generators (`bsp-after.sh`):
  `nickel export --format json config/operator-surfaces.ncl > config/operator-surfaces.json`,
  then `mantle __operator-contract --mode raw-descriptors|catalog|reference|workflow`
  into `config/operator-command-descriptors.json`, `config/operator-command-catalog.json`,
  `docs/generated/operator-command-reference.md`,
  `docs/generated/canonical-operator-workflow.md` (unchanged bytes).

Drift fixed on the new base:

1. Main itself failed the operator contract (`operator-flag-drift-main-2026-10-08.md`;
   BEFORE leg: `operator_contract_exit=1`, `operator_diagnostics` 17 passed / 1 failed).
   The cached offline export had added `source bundle export --cached-fetch` without
   policy. A standalone repair commit (policy row + regenerated files only) precedes the
   pin commit on this branch and was handed to the integrator for main.
2. The dev shell has no `python3` (`python_version_exit=127` in `focused-after-2026-10-08.md`);
   the earlier claim that the reader rail runs "inside the project dev shell" with bare
   `python3` was false on this host. Design and operator docs now provision Python from the
   flake-locked nixpkgs (`nix shell --inputs-from . nixpkgs#python3 --command python3 …`).
3. The `batched_resolution` requirement had no `r[impl …]` marker; `check` now carries it.

## Results by task

| Task | Evidence | Observed |
|---|---|---|
| T1.1 | `baseline-inventory-main-2026-10-08.md`, `baseline-focused-main-2026-10-08.md` | 185 root recipes; 131 files / 158 `url|source_url = "https://` literals; 131 files / 150 `hash|source_hash = "sha256-` literals (regex in the script). `refresh.rs`, `refresh_adapter.rs`, `project_resolve.rs` (`impl RefreshResolver for LiveResolver`) and `.cairn/specs/mantlepkgs-update-plans/spec.md` present. Baseline `crunch-project-core --lib refresh` 19 passed; `crunch-project --lib refresh` 6 passed; `crunch-project-core --lib` 161 passed; `mantle-portable-client-core --lib` 7 passed. The line-text `check-bootstrap-source-pins.rs` passes the three unmigrated recipes but already fails on main for the whole tree (`185 files, 140 fetch blocks, 2 issues`: factory `spec.source_hash`). |
| T1.2 | `focused-after-2026-10-08.md`, `recipe-rail-negatives-2026-10-08.md` | `bootstrap_pins` core tests 2 passed (round-trip, unknown field names known fields, missing hash, non-SRI hash); rail rejects an unknown TOML key (`pin fields must be exactly …`). |
| T1.3 | ADR 0086 + `adr/README.md` row | Boundary and plan-artifact decision recorded (status Proposed). |
| T2.1 | `focused-after-2026-10-08.md` | Core tests 2 passed; `crunch-project-core --lib` 163 passed (161 baseline + 2). |
| T2.2 | `recipe-rail-2026-10-08.md`, export comparison below | `generate_readers.py --check` and `--check-recipes` exit 0 (cmake + both picolibc recipes evaluated). |
| T2.3 | `cli-smoke-2026-10-08.md`, `focused-after-2026-10-08.md` | Round-trip unit test; unchanged picolibc world: two checks exit 0 with identical plans and an unchanged cache mtime (304, no body re-parse/re-write) on an empty `PATH` (`nickel` absent); pending CMake check exit 3 naming `3.31.8` → `4.4.4`. |
| T2.4 | `recipe-rail-negatives-2026-10-08.md`, core tests | Hard-coded URL, pre-bump URL, wrong hash and flat `fetchurl` each rejected (`evaluated source fetch URL, sha256 hash, or flat/tree mode differs from pin`, exit 1); stale derived reader rejected; unmutated control exit 0. |
| T3.1 | `cli-smoke-2026-10-08.md`, `host_bounds_use_parsed_https_authority` | Conditional cache (ETag) answered the second picolibc pass; CMake candidate prefetched through `LiveResolver::hash_url_content`; recomputed BLAKE3 plan digest equals stored digest. |
| T3.2 | `cli-smoke-2026-10-08.md` | Reviewed apply exit 0 `applied_sources:1`; only `bootstrap/pins/cmake.toml` and `bootstrap/pins/generated/cmake.json` changed (613/735 → 610/732 bytes); recipe bytes unchanged; re-check exit 0 `current`; picolibc apply `applied_sources:0` with bytes unchanged. |
| T3.3 | `cli-smoke-2026-10-08.md`, CLI tests 3 passed | Unresealed plan → `mutated or incomplete bootstrap pin plan`; changed TOML → `cmake: source pin changed since check`; resealed unknown source → `unknown source unknown-source`; resealed wrong hash → `cmake: upstream hash mismatch for source`; all exit 3 with root bytes unchanged; missing `--reviewed` exit 2; rollback and mixed-plan unit tests pass. |
| T4.1 | export comparison below, `cli-smoke-2026-10-08.md` | Migration is behaviour-neutral and the bump needs no Nickel edit. |

## Migrated recipe exports (main vs branch, `nickel export -I lib --format json`)

| Recipe | main bytes | branch bytes | sha256 (both) |
|---|---|---|---|
| `cmake-3.31.8-gcc10.ncl` | 7639 | 7639 | `a1f4d2bacaeabd97ac06778147d2d56eeec9d437a8ef7690b60497eee1e65172` |
| `picolibc-1.8.12-src.ncl` | 553 | 553 | `d631d8c5c36e71a0e441d2366f86c9afc2cff846c5eaeec6df9f8b489eced704` |
| `picolibc-1.8.12-diagnostic.ncl` | 4545 | 4545 | `4ca68a58e3a0f1145949957f03fc62f1daa37fb9a3bc5371526706652210848b` |

Byte-identical: consumers of `bootstrap/cmake-3.31.8-gcc10.ncl` (for example the
source-built fixed-point proof shell's host CMake) see the same derivation. After the
scratch 4.4.4 apply, the CMake export equals the pre-apply export with exactly
`3.31.8` → `4.4.4` and the old → new source hash substituted
(`cmake_export_equal_after_version_and_hash_substitution=yes`), so build phases,
inputs and builder are unchanged.

## Non-claims

No CMake 4.4.4 build, no claim that 4.4.4 works in the bootstrap graph (the bump ran
only in a scratch root; the repository stays on 3.31.8), no Picolibc lineage admission,
no legacy inline pin migration beyond CMake and Picolibc, and no durable multi-file
transaction.
