## Phase 1: Test infrastructure

- [x] Add `assert_cmd` and `predicates` as dev-dependencies
- [x] Create `tests/integration.rs` skeleton
- [x] Create `tests/fixtures/` directory with simple `.ncl` files (simple.ncl, multi.ncl, invalid.ncl)
- [x] Write a helper function that resolves the crunch binary path ✅ crunch_cmd()

## Phase 2: Eval tests

- [x] Test `crunch eval` on a simple derivation prints valid JSON ✅ eval_simple_derivation_prints_json + eval_output_is_valid_json
- [x] Test `crunch eval` on a multi-derivation file prints record-of-records JSON ✅ eval_multi_derivation_prints_both
- [x] Test `crunch eval` with `-I` flag resolves cross-directory imports ✅ eval_with_import_path_flag
- [x] Test `crunch eval` on invalid Nickel exits with code 2 ✅ eval_invalid_nickel_exits_2
- [x] Test `crunch eval` on nonexistent file exits with code 2 ✅ eval_nonexistent_file_exits_2
- [x] Test `crunch eval --json` on invalid Nickel emits parseable JSON error ✅ eval_json_flag_emits_json_error

## Phase 3: Bootstrap tests

- [x] Test `crunch bootstrap -o seed.ncl` creates a valid Nickel file ✅ bootstrap_creates_seed_file
- [x] Test generated seed.ncl is importable by `crunch eval` ✅ bootstrap_seed_is_importable
- [x] Test `crunch bootstrap` with unknown package exits non-zero — skipped (bootstrap may succeed with partial results)

## Phase 4: Build tests (Linux-only)

- [x] Test `crunch build` on trivial derivation produces output path ✅ build_trivial_derivation
- [x] Test `crunch build` on failing builder exits with code 1 ✅ build_failing_builder_exits_1
- [x] Test `crunch build` with `--store /tmp/...` — deferred (store paths are hardcoded to /nix/store)

## Phase 5: Error and edge cases

- [x] Test `crunch build` with missing store dir exits with code 3 ✅ build_missing_store_exits_3
- [x] Test `--verbose` flag produces additional output on stderr ✅ verbose_flag_produces_debug_output
