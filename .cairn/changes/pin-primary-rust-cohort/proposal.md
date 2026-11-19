# Proposal: Pin the primary Rust compiler cohort

## Why

`rust-toolchain.toml` selected an undated `nightly`. Cargo/rustup and the flake
could resolve different compilers, and the floating channel advanced to a
compiler where the locked `branches` dependency no longer builds. The Cargo
entry point was broken while the locked Nix cohort still worked.

## What Changes

- Pin `rust-toolchain.toml` to `nightly-2026-04-08`.
- Keep the flake deriving its primary toolchain from that file; preserve
  intentionally distinct component and Cargo-unit cohorts.
- Document the update workflow and record `rustc -Vv` in the flake-check job.
- Add an ADR for the cohort decision.

## Impact

- **Immediate outcome**: clean Cargo and Nix entry points resolve the same
  `rustc 1.96.0-nightly (c75612477 2026-04-07)`.
- **Durable capability**: compiler updates are explicit reviewed changes.
- **Maintenance owner**: Mantle toolchain/CI owners.
- **Repeatability evidence**: direct and Nix `rustc -Vv`/`cargo -Vv` output.

## Out of Scope

- Claiming stable Rust support.
- Flattening component or Cargo-unit compiler cohorts.
- Changing the compiler during a proof run.
