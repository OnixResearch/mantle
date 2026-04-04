## Phase 1: Extract and test error formatting

- [ ] Move RunError to src/errors.rs with Display impl and exit_code() method
- [ ] Add tests: each RunError variant maps to correct exit code
- [ ] Add tests: Display output includes actionable information

## Phase 2: Build failure message improvement

- [ ] Parse bwrap error to extract builder stderr
- [ ] Show the failing builder command in error output
- [ ] Add common fix suggestions (missing bwrap, store permissions, missing inputs)
- [ ] Test error extraction with sample bwrap error strings

## Phase 3: Bootstrap testability

- [ ] Extract nix resolution into a trait/closure parameter
- [ ] Add tests with mock resolver: success path produces valid seed.ncl
- [ ] Add tests with mock resolver: failure path reports package name
- [ ] Test seed.ncl output format: valid Nickel, correct store paths

## Phase 4: Structured error output

- [ ] Add `--json` global flag to clap Args
- [ ] Emit JSON error object when flag is set
- [ ] Test JSON output parsing for each error variant
