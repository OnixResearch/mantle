# Design: Align hello-world builder assertion

## Context

`examples/hello-world.ncl` currently defines the example derivation with
`builder = "/bin/sh"`.

`tests/integration_build.rs::eval_hello_world_with_seed` still asserted that
`drv.builder.contains("bash")`. That assertion no longer describes the example
under test, so the integration test reports a fake regression.

## Goals / Non-Goals

**Goals:**

- make the hello-world eval round-trip assertion match the current example
- keep the change limited to test validation
- preserve signal from the broader lib/tests rerun

**Non-Goals:**

- change `examples/hello-world.ncl`
- change derivation evaluation or glue behavior
- fold this test-alignment fix into the separate self-build PATH change

## Decisions

### 1. Assert the exact builder contract

**Choice:** replace the stale substring assertion with `assert_eq!(drv.builder,
"/bin/sh")`.

**Rationale:** the example already uses an exact builder path. The test should
assert the real contract, not a looser historical implementation detail.

### 2. Keep validation focused on the existing regression surface

**Choice:** prove the change with the targeted integration test and the broader
lib/tests rerun.

**Rationale:** the targeted test confirms the exact stale assertion is fixed,
while the broader rerun confirms the branch still passes the normal validation
path.

## Verification

- Targeted command: `cargo test -p crunch --test integration_build eval_hello_world_with_seed -- --nocapture`
  - expected result: `test eval_hello_world_with_seed ... ok`
- Broader command: `cargo test -p crunch -p crunch-pipeline --lib --tests`
  - expected result: `tests/integration_build.rs` reports `9 passed; 0 failed`
  - expected result: the command exits successfully
