## Phase 1: Format

- [ ] [serial] Specify transcript block types, expected-error semantics, hidden setup, and output normalization.
- [ ] [parallel] Add fixture transcripts for a trivial CLI workflow and an expected failure.
- [ ] [parallel] Define isolated store/state defaults and in-place opt-in marker.

## Phase 2: Runner and gates

- [ ] [depends:Phase 1] Implement transcript parser/runner or CLI subcommand.
- [ ] [parallel] Add tests for expected success, expected failure, hidden setup, and state isolation.
- [ ] [parallel] Add a maintained quality-gate entrypoint for fast transcripts.
- [ ] [depends:Phase 2] Convert one operator doc or release-proof walkthrough into a checked transcript.
