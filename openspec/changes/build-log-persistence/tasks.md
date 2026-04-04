## Phase 1: Understand bwrap output capture

- [ ] Read `BubblewrapBuildService` spawn logic to find where stdout/stderr are handled
- [ ] Determine whether output capture requires a vendored patch or can be done at the orchestrator level
- [ ] Document the chosen approach in this file before proceeding

## Phase 2: Capture build output

- [ ] Add `log: Option<String>` field to `BuildOutcome`
- [ ] Capture stdout/stderr from the build sandbox into `BuildOutcome.log`
- [ ] On build failure, include captured output in the error (not just bwrap's error message)

## Phase 3: Persist logs

- [ ] Write `<drv-hash>.log` to `$CRUNCH_LOG_DIR` on every build (success and failure)
- [ ] Include metadata header in log file (derivation name, timestamp, exit status)
- [ ] Test log file creation and content for a successful build
- [ ] Test log file creation and content for a failed build

## Phase 4: CLI display

- [ ] Display full build log on failure (before error summary)
- [ ] Display build log on success when `--verbose` is set
- [ ] Suppress build log on success without `--verbose` (current behavior)

## Phase 5: `crunch log` subcommand

- [ ] Add `Log` variant to CLI `Command` enum
- [ ] Implement log lookup by derivation hash or store path
- [ ] Implement `--list` to show stored logs with derivation names and dates
- [ ] Test `crunch log` with a known log file
