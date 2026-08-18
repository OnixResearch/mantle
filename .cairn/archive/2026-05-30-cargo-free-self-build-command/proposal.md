## Why

Mantle can already prove a bounded Cargo-free fixed point through `scripts/prove-cargo-free-fixed-point.rs`, but operators still have to run proof machinery to get a produced Mantle binary. The next usable step is a first-class CLI build path that packages one Cargo-free topology execution as `mantle self-build --cargo-free --out <dir>`.

## What Changes

- Add `mantle self-build --cargo-free --out <dir>` as a bounded self-build mode.
- Run the existing native Rust topology rail with `--no-cargo-oracle` and a failing Cargo guard.
- Write the produced Mantle binary plus receipt, source digest, command streams, Cargo guard status, and non-claims to the output directory.
- Add positive and negative CLI coverage for the new mode.

## Impact

- **Files**: `src/main.rs`, new CLI support module, `tests/*`, `cairn/specs/rust-package-planning/spec.md` via this change.
- **Testing**: focused CLI tests for a tiny Mantle fixture, Cargo guard negative test, Cairn validation/gates, and a real Mantle self-build smoke when practical.
