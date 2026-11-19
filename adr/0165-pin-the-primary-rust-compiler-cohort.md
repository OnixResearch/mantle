# 0165. Pin the primary Rust compiler cohort to a dated nightly

Status: Proposed

## Context

`rust-toolchain.toml` selected the undated `nightly` channel. The flake
derived its primary toolchain from that file, while a direct rustup/Cargo
workflow resolved the same floating channel independently. The floating
channel advanced to a compiler where the locked `branches` dependency no
longer builds (`core::intrinsics::abort` disappeared), breaking the Cargo
entry point even though the locked Nix cohort remained usable.

## Decision Drivers

- One reviewed compiler for Cargo, rustup, and the primary Nix shell.
- Explicit update decisions instead of a moving target.
- Preserve intentionally distinct component and Cargo-unit cohorts.
- Keep Casita's minimum Rust version and cross-target requirements enforced.

## Decision

The primary cohort is pinned to `nightly-2026-04-08`
(`rustc 1.96.0-nightly (c75612477 2026-04-07)`) in
`rust-toolchain.toml`. The flake continues to derive its primary
`rustToolchain` from that file, so Cargo and Nix select the same date.
Component and Cargo-unit compilers keep their own recorded cohorts.

Updates are reviewed changes that record `rustc -Vv`, run the workspace
compile/tests and focused gates, check Casita's `rust-version` and the listed
wasm/Apple targets, and avoid changing compilers during a proof run.

## Consequences

- A clean Cargo entry point and the checked-in Nix shell resolve the same
  compiler version and commit.
- New nightly features are no longer available implicitly; adopting one is an
  explicit update.
- The workaround for the older compiler/lock interaction is retained until a
  compatible cohort update is validated.
