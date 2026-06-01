## Why

Mantle examples currently have only partial automated coverage. A few examples have evaluation tests, one fetcher build smoke exists, and the heavy real-crate build is ignored. Many examples can drift in Nickel shape, conversion behavior, build output layout, or runtime output without a deterministic rail catching it.

## What Changes

- Add a tiered validation matrix for examples: evaluate all eligible Nickel examples, build fast/offline smoke examples in temp store/state roots, execute built outputs when practical, and leave heavyweight examples behind explicit ignored tests or scripts.
- Add both positive and negative checks. Valid examples must evaluate/build/execute as promised; malformed or intentionally failing examples must fail with expected diagnostics.
- Keep build tests hermetic: temp store/state, no ambient store mutation, explicit bwrap/Linux capability skips, and no hidden host tools beyond documented capabilities.

## Impact

- **Files**: `tests/examples_eval.rs`, `tests/examples_build.rs`, possible new `tests/examples_inventory.rs`, example fixture files, and example docs.
- **Testing**: focused examples eval/build tests, output-execution assertions, negative diagnostics tests, `cairn validate --root .`, and task-gate evidence.
