# Tasks: Add hermeticity modes and reporting

## Phase 1: Mode plumbing

- [x] Add a shared `HermeticityMode` enum (or equivalent) for build-entry commands
- [x] Add `--strict-hermetic` to `crunch build`
- [x] Add `--strict-hermetic` to `crunch self-build`
- [x] Thread hermeticity mode through pipeline and self-build config without changing it in transit

## Phase 2: Audit model and reporting

- [x] Define typed hermeticity audit events for degraded execution facts
- [x] Extend human-readable build output with hermeticity mode and audit summaries
- [x] Extend JSON build output with hermeticity mode and audit events
- [x] Add tests for clean and degraded report rendering

## Phase 3: Validation

- [ ] Re-read the touched CLI and build-pipeline specs against the proposed mode/report shape before implementation starts
- [x] Run `openspec validate add-hermeticity-modes-and-reporting`
