## Context

`materialize_rust_source_provider` currently computes the route plan path from the recipe parent and fixed `rust-source-plan.ncl` name. That keeps the first route simple but blocks a parallel musl-host route. The next native-closure proof path needs the Rust provider host triple to be `x86_64-unknown-linux-musl` so the source-root musl toolchain can satisfy host native closure members.

## Design

### Functional core

Split plan path selection into an explicit request type: a route plan may be caller-supplied, otherwise the existing recipe-relative default is used. The core still validates the route plan through `validate_rust_source_provider_bootstrap_plan` before any source build phase.

### CLI shell

`mantle bootstrap rust-source-provider` gains `--route-plan <path>`. The default remains `bootstrap/rust-source-plan.ncl`, preserving current GNU-host behavior. The shell passes the selected route plan path into materialization; imports and smoke behavior remain unchanged.

### Musl-host plan

`bootstrap/rust-source-musl-host-plan.ncl` mirrors the existing source route and final version, but sets both host and target triples to `x86_64-unknown-linux-musl` and points both host/target rustlib final outputs at the musl rustlib path. This is route metadata only; it does not claim the real provider is built.

### Claim boundary

A valid musl-host route plan is not a materialized Rust provider and does not retire any non-claim. It only unblocks a future materialization attempt using the existing source-root musl native closure.
