## Why

Native Rust topology now reaches `nix-compat-derive`, but its generated rustc args still hard-code `--edition 2021`. The crate manifest declares edition `2024`, and rustc rejects its let-chain syntax when Mantle invokes it with the wrong edition. Cargo's unit graph is no longer the execution orchestrator here, so the native planner must carry edition facts itself.

## What Changes

- Record each native target's manifest package edition, including `edition.workspace = true`, in native package/target facts.
- Use the recorded edition when generating native target, native host, and helper dev-dependency rustc args.
- Default absent `package.edition` to Cargo's documented 2015 behavior instead of hard-coded 2021.
- Add focused positive and negative tests for declared and missing edition behavior.
- Re-run the self-probe to verify movement past the Rust 2024 let-chain blocker.

## Impact

- **Files**: `src/rust_plan.rs`, `cairn/specs/rust-package-planning/spec.md`, change evidence/tasks.
- **Testing**: focused rust-plan tests, clean self-probe, `cairn validate`, `git diff --check`.
