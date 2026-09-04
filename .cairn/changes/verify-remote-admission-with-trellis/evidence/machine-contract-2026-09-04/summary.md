# Machine-contract evidence

The first five generation attempts exposed contract defects before generation completed.

The defects were:

- Missing lowercase BLAKE3 freshness placeholders.
- A positive fixture outside an admitted review path.
- A producer path outside `src`, `crates`, or `tools`.
- A missing `machine-artifact-public` marker.
- An unsupported JSON Schema `pattern` field.
- A wrong expected issue class for the Valence role mutation.

Later validation found that a cargo-script function cannot own a contracted Rust DTO. The report type now lives in `crunch-release-core`.

The final generation reported `machine schema contract generation: PASS (33 contracted, 67 classified)`.

The final check reported `machine schema contract check: PASS (33 contracted, 67 classified)`.

All failed logs remain in this directory. The final generated contract, positive fixture, negative fixture set, and freshness bindings are checked in.
