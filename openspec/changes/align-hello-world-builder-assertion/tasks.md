## Phase 1: Align the stale assertion

- [x] Replace the stale `drv.builder.contains("bash")` assertion in `tests/integration_build.rs::eval_hello_world_with_seed` with the current `/bin/sh` contract
- [x] Keep the scope to test alignment only; do not change `examples/hello-world.ncl` or runtime code

## Phase 2: Verify the aligned contract

- [x] Run `cargo test -p crunch --test integration_build eval_hello_world_with_seed -- --nocapture` under the documented Cargo environment and confirm the targeted test passes
- [x] Run `cargo test -p crunch -p crunch-pipeline --lib --tests` under the documented Cargo environment and confirm the command exits successfully and `tests/integration_build.rs` reports `9 passed; 0 failed`

## Validation

- [x] Run `openspec validate align-hello-world-builder-assertion`
