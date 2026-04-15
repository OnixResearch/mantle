# Align hello-world builder assertion

## Why

`tests/integration_build.rs::eval_hello_world_with_seed` drifted from the
current `examples/hello-world.ncl` contract.

The example now sets `builder = "/bin/sh"`, but the test still required the
builder string to contain `bash`. That stale expectation keeps a broad
`cargo test -p crunch -p crunch-pipeline --lib --tests` rerun from staying
focused on real regressions.

## What Changes

- align the hello-world eval round-trip assertion with the current `/bin/sh`
  builder contract
- keep the validation scope on test alignment only; no runtime behavior change
- preserve broader self-build PATH hardening as a separate OpenSpec change

## Capabilities

### Modified Capabilities

- `hello-world-example-validation`: the integration test for the hello-world
  eval round-trip matches the current example builder contract

## Impact

- **Files**: `tests/integration_build.rs`
- **Behavior**: no product behavior change; test expectation only
- **Testing**: rerun the targeted integration test and the broader lib/tests
  command under the documented Cargo environment
