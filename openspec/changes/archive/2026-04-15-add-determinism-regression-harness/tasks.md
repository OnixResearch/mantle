# Tasks: Add determinism regression harness

## Phase 1: Scope alignment

- [x] Re-read the touched build-pipeline spec against the planned harness cases before implementation

## Phase 2: Harness shape

- [x] Build a harness that reruns selected builds under varied ambient host state
- [x] Vary at least `HOME`, `PATH`, `USER`, `TZ`, `LANG`, `TMPDIR`, cwd, and umask
- [x] Compare output digests and hermeticity audit facts across successful runs
- [x] Cover a known strict-mode blocker and compare blocker class stability across runs
- [x] Verify the strict-mode blocker does not silently degrade into a weaker success or audit-only path under ambient variation

## Phase 3: Coverage

- [x] Cover at least one normal derivation
- [x] Cover at least one fetcher-rooted build
- [x] Cover at least one self-build-friendly path
- [x] Keep the initial matrix small enough for regular repo validation by limiting it to targeted ambient cases and targeted test filters
- [x] Wire the harness into automated Rust test coverage

## Phase 4: Validation

- [x] Run targeted Rust tests for the new determinism coverage and keep the command transcript in `validation.md`
- [x] Run `openspec validate add-determinism-regression-harness` and keep the command transcript in `validation.md`
