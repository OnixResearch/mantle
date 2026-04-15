## ADDED Requirements

### Requirement: Hello-world eval validation matches the example builder contract

The hello-world eval round-trip validation MUST assert the builder contract that
`examples/hello-world.ncl` actually exports.

#### Scenario: Targeted hello-world eval rerun passes with the current builder

- GIVEN `examples/hello-world.ncl` defines `builder = "/bin/sh"`
- WHEN a contributor runs `cargo test -p crunch --test integration_build eval_hello_world_with_seed -- --nocapture` under the documented Cargo environment
- THEN `tests/integration_build.rs::eval_hello_world_with_seed` passes
- AND the output contains `test eval_hello_world_with_seed ... ok`

#### Scenario: Broader lib/tests rerun keeps the aligned assertion green

- GIVEN the hello-world builder assertion is aligned with the current example contract
- WHEN a contributor runs `cargo test -p crunch -p crunch-pipeline --lib --tests` under the documented Cargo environment
- THEN the command exits successfully
- AND `tests/integration_build.rs` reports `9 passed; 0 failed`
