## ADDED Requirements

### Requirement: Selectable Rust source provider route plans

r[rust_package_planning.source_built_toolchain_closure.selectable_rust_source_route] Mantle MUST let Rust source provider materialization select an explicit validated route plan while preserving the existing GNU-host route as the default.

#### Scenario: Default route remains GNU host

GIVEN an operator runs `mantle bootstrap rust-source-provider` without `--route-plan`
WHEN Mantle prepares the Rust source provider materialization
THEN it MUST use the existing recipe-relative `rust-source-plan.ncl` route.
AND that route MUST continue to advertise `host_triple = x86_64-unknown-linux-gnu`.

#### Scenario: Explicit musl-host route validates

GIVEN an operator passes `--route-plan bootstrap/rust-source-musl-host-plan.ncl`
WHEN Mantle loads the Rust source provider route
THEN the route plan MUST validate with `host_triple = x86_64-unknown-linux-musl`.
AND the plan MUST keep `target_triple = x86_64-unknown-linux-musl`.
AND the plan MUST forbid prebuilt Rust.

#### Scenario: Route selection is metadata only

GIVEN the musl-host route plan validates
WHEN no materialized provider output exists
THEN Mantle MUST NOT report a source-built Rust provider claim from the route plan alone.
AND it MUST NOT retire `not-source-built-toolchain-closure`.
