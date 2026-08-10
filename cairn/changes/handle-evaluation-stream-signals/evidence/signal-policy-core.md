# Signal policy core evidence

Question: Is first-versus-repeated interruption policy deterministic and free of process effects?

Inspected evidence:

- `crates/crunch-evaluation-stream-core/src/projection.rs`
- `crates/crunch-evaluation-stream-core/src/tests.rs`
- Pueue task `17181`

Results:

- Zero observations return no action.
- The first observation requests cooperative cancellation.
- Every later observation forces interruption.
- Eighteen focused core tests passed.
- The core compiled for `wasm32-unknown-unknown`.
- The core imports no signal, process, async, JSON, or output API.

Decision: I2 is complete. The CLI shell can now apply supplied signal observations without owning policy logic.

Owner: `handle-evaluation-stream-signals`.

Next action: Add platform listeners and connect them to the pure policy.
