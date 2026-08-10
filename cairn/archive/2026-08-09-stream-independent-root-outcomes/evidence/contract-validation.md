# Evaluation stream contract validation

Question: Does the stream contract reject malformed input and admit the two supported Phase 1 fixtures?

Inspected evidence:

- `docs/evaluation-stream-contract.md`
- `adr/0075-stream-complete-root-outcomes-with-canonical-summaries.md`
- `schemas/evaluation-stream/mantle-evaluation-stream-v1.schema.json`
- `schemas/evaluation-stream/fixtures/`
- `scripts/check-evaluation-stream-contract.rs`
- Pueue tasks `16169` and `16173`

Results:

- The checker self-test passed.
- The contract check passed two positive fixtures.
- The contract check rejected five negative fixtures at their target boundaries.
- The negative classes are unknown version, unknown kind, duplicate field, oversized field, and malformed record.
- Rust formatting passed for the contract checker.
- `git diff --check` passed.
- Cairn validation passed after the contract changes.
- The proposal, design, and tasks gates passed after the contract changes.

Decision: The v1 contract is complete enough for pure outcome-core implementation. Pipeline dispatch remains unchanged.

Owner: `stream-independent-root-outcomes`.

Next action: Implement the pure outcome and projection core before task I11 changes pipeline dispatch.
