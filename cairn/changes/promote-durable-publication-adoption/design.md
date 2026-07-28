# Design

## Context

The reviewed Mantle implementation commit is `d1f3d6d96e2b0d9cd8497cd89b8e5a93d4a7dfaf`. It adopts durable publication RID `rad:z3tAR4For7qw8ZirkJzoDw1VNDDLM` at revision `951c27f59003cea9bfdb40ed4d89653d50fada1f`.

## Decisions

### Require both local and producer ancestry

The promotion candidate must descend from the reviewed Mantle commit. Every later commit must be reviewed and limited to the named Cairn planning and evidence roots until promotion. Validation binds one exact candidate hash. Before mutation, remote Onix Core `main` must contain `b8387cd7d59fa3b0d4ea67646352dd27c4f7d7ed`.

### Re-evaluate remote state before push

Fetch Mantle `origin`, resolve `origin/main`, and prove fast-forward ancestry immediately before mutation. A moved or divergent target stops promotion. Force-push and pull-request paths are forbidden.

### Gate promotion with focused accepted evidence

Run the shared-backend tests, full Mantle binary tests sequentially, package build, version probe, formatting, product-owned Clippy, Tiger Style, Nickel evidence, focused Nix adoption check, Cairn validation, and focused traceability.

`restore-durable-publication-broad-validation` owns unrelated source-filter, blocker-inventory, vendored-dependency, and Octet failures. Those failures remain visible but do not reinterpret accepted focused evidence.

### Keep cleanup in a dependent lifecycle

`retire-durable-publication-worktree` owns post-archive cleanup. Promotion does not try to prove deletion after its own archive.

## Failure handling

Any producer-order, ancestry, check, push, or remote-verification failure stops the workflow without weakening policy or changing canonical history.
