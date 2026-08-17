# Proposal: Bind native Rust artifacts by unit variant identity

## Problem

Mantle's native Rust topology still collapses produced artifacts by package ID in places where Cargo distinguishes unit variants. The latest clean self-probe advanced past the `crunch_glue` transitive search-path bug, then failed while compiling `crunch-build`:

```text
error E0463: can't find crate for `crunch_store`
 --> ./crates/crunch-build/src/ca_mapping.rs:5:9

error: found crates (`snix_castore` and `snix_castore`) with colliding StableCrateId values
 --> ./crates/crunch-build/src/fod.rs:5:5
```

Probe evidence shows duplicate same-package producers before the failed unit:

- `crunch-build`: unit `30` succeeded, unit `6` failed.
- `crunch-store`: units `15` and `39` both produced `libcrunch_store.rlib`.
- `snix-castore`: units `20` and `512` both produced `libsnix_castore.rlib`; custom-build units `519` and `613` also both ran.

Package-keyed binding can select the wrong direct producer, while global search-path history can expose two same-crate variants to rustc and trigger StableCrateId collisions.

## Change

Make native topology artifact binding unit-variant-aware. Direct dependency placeholders and produced artifact maps must identify the selected producer unit, not only the package ID. Search paths should be scoped to the selected producer closure for the consumer, not every previously produced artifact directory.

## Success criteria

- Same-package unit variants remain distinct in native dependency planning and execution.
- A consumer binds its `--extern` to the exact selected producer variant from the Cargo unit graph/native unit graph.
- Rustc search paths include only the selected producer closure needed by that consumer, avoiding unrelated same-crate variants.
- Ambiguous or missing producer variants fail before rustc with deterministic blockers.
- A clean self-probe advances past the `crunch_store` / `snix_castore` frontier or reaches a later deterministic blocker.
