## Why

After proc-macro host artifact binding, the clean self-probe gets past `darling@0.20.11` and reaches `derive_builder_core@0.20.2`. Direct rustc then fails because a dependency warning is promoted to an error by the dependency crate's lint settings. Cargo normally passes `--cap-lints allow` for non-local dependencies, so Mantle needs bounded cap-lints parity for registry/git dependency units.

## What Changes

- Add deterministic `--cap-lints allow` to rustc args for non-local source units (`registry` and `git`).
- Preserve path/workspace source lint behavior by not capping local path units.
- Apply the same source-kind rule to target and host units.
- Record whether the self-probe moves past the `derive_builder_core` lint blocker.

## Impact

- **Files**: `src/rust_plan.rs`, Cairn change/evidence files.
- **Testing**: focused `rust_plan::` tests, clean self-probe, `cairn validate --root .`, `git diff --check`.
