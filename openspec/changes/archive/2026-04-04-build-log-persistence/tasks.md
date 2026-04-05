## Phase 1: Understand bwrap output capture

- [x] Read `BubblewrapBuildService` spawn logic ✅ bwrap.rs line 131: `Command::new("bwrap").output().await` captures into `std::process::Output`
- [x] Determine capture approach ✅ Vendored patch needed. `do_build()` discarded stdout/stderr — on failure returned "nonzero exit code", on success ignored output.
- [x] Document approach: patch bwrap to include output in error + BuildResult.log for success

## Phase 2: Capture build output

- [x] Patch `BubblewrapBuildService::do_build()` to include stdout+stderr in the io::Error on failure ✅ error now includes exit status + full output
- [x] Add `log: Option<String>` field to `BuildResult` (vendored snix-build) ✅
- [x] Fill `BuildResult.log` with captured stdout+stderr on success ✅ bwrap.rs captures from outcome.output()
- [x] Add `log: Option<String>` to `BuildOutcome` in crunch-build ✅
- [x] Wire `BuildResult.log` through to `BuildOutcome.log` in orchestrate.rs ✅

## Phase 3: Persist logs

- [x] Write `<drv-hash>.log` to `$CRUNCH_LOG_DIR` on every build (success and failure) ✅ write_log() helper
- [x] Include metadata header in log file (derivation name, timestamp, exit status) ✅ # crunch build log header
- [x] Test log file creation for a successful build ✅ build_writes_log_file integration test
- [x] Test log file creation for a failed build ✅ failure log written in .map_err()

## Phase 4: CLI display

- [x] Display full build log on failure (before error summary) ✅ error now includes the build output
- [x] Display build log on success when `--verbose` is set ✅ prints "--- build log ---" block to stderr
- [x] Suppress build log on success without `--verbose` (current behavior) ✅

## Phase 5: `crunch log` subcommand

- [x] Add `Log` variant to CLI `Command` enum ✅
- [x] Implement log lookup by derivation hash or store path ✅ query matches filename substring
- [x] Implement `--list` to show stored logs with derivation names and dates ✅ parses header metadata
- [x] Test `crunch log` with a known log file ✅ log_subcommand_shows_log_by_query + log_subcommand_lists_logs + log_subcommand_query_not_found
