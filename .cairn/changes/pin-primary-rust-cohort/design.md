# Design: Pin the primary Rust compiler cohort

## Goal and scope

One primary host/contributor compiler for Cargo, rustup, and the checked-in Nix
shell. Component and Cargo-unit compilers keep their independently recorded
cohorts.

## Decision

| Boundary | Decision | Rejected alternative |
| --- | --- | --- |
| Primary channel | Dated `nightly-2026-04-08` in `rust-toolchain.toml` | Floating `nightly` |
| Nix derivation | `fromRustupToolchainFile ./rust-toolchain.toml` | A separate hardcoded Nix date |
| Update flow | Reviewed change with `rustc -Vv`, workspace checks, targets, and Casita minimum version | Implicit channel advance |
| Evidence | Direct and Nix compiler output recorded; CI prints `rustc -Vv` | Ambiguous "nightly" label |

## Failure behavior

A clean Cargo or Nix entry point that resolves a different compiler is a
toolchain update, not an accepted ambient difference. A compiler change during
a proof run is out of scope.

## Verification

Compare direct `rustc -Vv` and `nix develop -c rustc -Vv`; require the same
version and commit. Run workspace compile/tests before accepting a future
cohort update.
