## Phase 1: Extract and test error formatting

- [x] Move RunError to src/errors.rs with Display impl and exit_code() method ✅ 2m (started: 2026-04-04T18:57Z -> completed: 2026-04-04T18:59Z)
- [x] Add tests: each RunError variant maps to correct exit code ✅ (included in errors.rs)
- [x] Add tests: Display output includes actionable information ✅ (included in errors.rs)

## Phase 2: Build failure message improvement

- [x] Parse bwrap error to extract builder stderr ✅ 1m (started: 2026-04-04T18:59Z -> completed: 2026-04-04T19:00Z)
- [x] Show the failing builder command in error output ✅ (suggestions point to log dir + verbose flag)
- [x] Add common fix suggestions (missing bwrap, store permissions, missing inputs) ✅ (6 patterns: nonzero-exit, namespace, source-not-found, fod-mismatch, not-linux, output-missing)
- [x] Test error extraction with sample bwrap error strings ✅ (9 tests covering all patterns + generic fallback)

## Phase 3: Bootstrap testability

- [x] Extract nix resolution into a trait/closure parameter ✅ 2m (started: 2026-04-04T19:01Z -> completed: 2026-04-04T19:03Z)
- [x] Add tests with mock resolver: success path produces valid seed.ncl ✅ (6 tests in bootstrap::tests)
- [x] Add tests with mock resolver: failure path reports package name ✅ (resolve_failure_reports_package)
- [x] Test seed.ncl output format: valid Nickel, correct store paths ✅ (generate_seed_valid_structure, sanitizes_hyphens, empty_packages)

## Phase 4: Structured error output

- [x] Add `--json` global flag to clap Args ✅ 1m (started: 2026-04-04T19:03Z -> completed: 2026-04-04T19:04Z)
- [x] Emit JSON error object when flag is set ✅ (wired in main() error handler)
- [x] Test JSON output parsing for each error variant ✅ (json_format_eval, json_format_build, json_format_internal, json_escapes_special_chars)
