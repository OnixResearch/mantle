# Tasks: Add hermeticity modes and reporting

## Phase 1: Mode plumbing

- [ ] Add a shared `HermeticityMode` enum (or equivalent) for build-entry commands
- [ ] Add `--strict-hermetic` to `crunch build`
- [ ] Add `--strict-hermetic` to `crunch self-build`
- [ ] Thread hermeticity mode through pipeline and self-build config without changing it in transit

## Phase 2: Audit model and reporting

- [ ] Define typed hermeticity audit events for degraded execution facts
- [ ] Extend human-readable build output with hermeticity mode and audit summaries
- [ ] Extend JSON build output with hermeticity mode and audit events
- [ ] Add tests for clean and degraded report rendering

## Phase 3: Validation

- [ ] Re-read the touched CLI and build-pipeline specs against the proposed mode/report shape before implementation starts
- [ ] Run `openspec validate add-hermeticity-modes-and-reporting`
