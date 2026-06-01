# Tasks

## Offline fixtures

- [x] [serial] Add or generate deterministic local fixtures for file, tarball, and git fetcher examples, keeping fixture contents small and reviewable. r[examples.offline_fetcher_fixtures] Evidence: `tests/examples_build.rs` generates small file, tarball, and test-owned git repository fixtures; `evidence/implementation-validation-2026-06-01.md` records focused offline tests passing.
- [x] [serial] Add catalog entries and docs that pair each real-network cookbook fetcher example with its offline validation fixture. r[examples.offline_fetcher_fixtures] Evidence: `examples/catalog.ncl` adds offline fixture rails for public fetcher examples, `examples/README.md` maps each real-network example to the offline rail, and `tests/examples_inventory.rs` rejects missing offline rails.
- [x] [serial] Add positive offline build tests for `fetchurl`, `fetchTarball`, and `fetchGit` using temp store/state roots and no external network. r[examples.offline_fetcher_fixtures] Evidence: `evidence/implementation-validation-2026-06-01.md` records pueue task `255` with `offline_fetchurl_fixture_builds_without_network`, `offline_fetch_tarball_fixture_builds_without_network`, and `offline_fetchgit_fixture_builds_without_network` passing.

## Negative fixed-output behavior

- [x] [serial] Add wrong-hash fixture coverage for file, tarball, and git fetchers where the helper supports fixed-output validation. r[examples.fixed_output_negative_cases] Evidence: `tests/examples_build.rs::offline_fetcher_wrong_hashes_fail_closed` covers wrong hashes for generated file, tarball, and git fixtures; pueue task `255` records it passing.
- [x] [serial] Add a temp-copy `--fix` or repair-workflow test that verifies the corrected hash path or diagnostic without mutating checked-in examples. r[examples.fixed_output_negative_cases] Evidence: `tests/examples_build.rs::fix_flag_updates_temp_fetchurl_fixture_hash` uses a temp fixture, asserts `fixed:`, verifies the wrong hash is replaced by the computed hash, and reruns the repaired fixture; pueue task `255` records it passing.
- [x] [serial] Assert failed fixed-output examples do not persist or report successful outputs. r[examples.fixed_output_negative_cases] Evidence: `assert_fod_failure` checks non-zero status, empty stdout, fixed-output diagnostics, and an empty temp store for each wrong-hash fixture; pueue task `255` records the negative test passing.

## Verification

- [x] [serial] Run focused fetcher example tests and record output, including positive offline and negative mismatch cases. r[examples.offline_fetcher_fixtures] r[examples.fixed_output_negative_cases] Evidence: `evidence/implementation-validation-2026-06-01.md` records pueue task `255` with `examples_build offline_fetch`, `examples_build fix_flag_updates_temp_fetchurl_fixture_hash`, and `examples_inventory` all passing.
- [x] [serial] Run `cairn validate --root .` and the tasks gate, then archive only after completed tasks cite durable evidence. r[examples.offline_fetcher_fixtures] r[examples.fixed_output_negative_cases] Evidence: `evidence/implementation-validation-2026-06-01.md` records `cairn validate --root .` with `"valid": true`, tasks gate with `"verdict": "PASS"`, `rustfmt --check`, and `git diff --check`.
