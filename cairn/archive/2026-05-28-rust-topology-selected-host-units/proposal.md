## Why

After bounded `rustc-link-lib` metadata parsing, the clean native rust-plan self-probe advances to `jiff-static@0.2.23`, which fails as a host proc-macro unit with unresolved `quote` and `syn` imports. `jiff-static` is not present in the Cargo-selected unit graph for the current Mantle build, so native topology should not schedule it at all. Planning every manifest-visible host target fabricates unselected host work and creates false frontiers.

## What Changes

- Derive native host units only from Cargo-selected host units.
- Keep selected proc-macro/custom-build host units, same-package build-script ordering, metadata dependencies, and target host-artifact consumers intact.
- Add positive and negative tests proving selected host units stay planned while unselected proc-macro packages are ignored.

## Impact

- **Files**: `src/rust_plan.rs`, `cairn/specs/rust-package-planning/spec.md`
- **Testing**: focused native host planning tests, clean native rust-plan self-probe, `cairn validate`, Cairn tasks gate.
