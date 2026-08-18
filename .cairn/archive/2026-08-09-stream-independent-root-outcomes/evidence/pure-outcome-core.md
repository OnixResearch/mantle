# Pure outcome core evidence

Question: Does Phase 2 provide deterministic and bounded root-outcome decisions without evaluator, worker, JSON, output, or process effects?

Inspected evidence:

- `crates/crunch-evaluation-stream-core/`
- `Cargo.toml`
- `Cargo.lock`
- ADR 0075
- Pueue tasks `16383`, `16396`, and `16398`

Approach registry:

- A separate `no_std` core crate is validated. It gives the rules one bounded authority surface.
- A `crunch-pipeline` module was rejected. It would mix outcome decisions with worker and build orchestration.
- A `crunch-eval` module was rejected. Build references and process status are outside evaluator authority.

Results:

- Sixteen focused tests passed.
- Five property tests cover terminal exclusivity, complete accounting, checked bounds, canonical order, schedule independence, and equivalent replay.
- Positive tests cover success, partial results, shared fatal errors, cancellation, coordinator loss, projection values, and process status.
- Negative tests reject empty roots, duplicate roots, invalid identities, oversized fields, missing references, invalid transitions, duplicate terminals, and incomplete summaries.
- The crate compiled for `wasm32-unknown-unknown`.
- Focused Clippy passed with warnings denied.
- The Tiger Style check passed.
- Rust formatting passed.
- The Phase 1 stream contract self-test and fixture check passed.
- `git diff --check` passed.
- Cairn validation and the proposal, design, and tasks gates passed.

Adversarial audit:

- Completion order does not affect canonical summaries or identities.
- Cancellation has precedence over partial success.
- Late success cannot replace a cancellation result.
- Shared fatal errors classify started and pending roots differently.
- Identity framing has fixed domains, field order, byte lengths, and known-answer tests.
- Production core modules do not import `std`, JSON, async, files, clocks, environment state, or process APIs.

Decision: Phase 2 is complete. Pipeline dispatch remains unchanged.

Owner: `stream-independent-root-outcomes`.

Next action: Integrate the core at task I11 without weakening bounded worker concurrency.
