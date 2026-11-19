# Rust toolchain cohort

Mantle pins one primary host/contributor compiler:

```text
nightly-2026-04-08
rustc 1.96.0-nightly (c75612477 2026-04-07)
```

`rust-toolchain.toml` is the source of truth for the Cargo entry point. The
flake derives its primary `rustToolchain` from that file through rust-overlay,
so clean Cargo and Nix entry points select the same date. Component and
Cargo-unit compilers remain intentionally separate where the flake records
their own cohort; this pin does not flatten those identities.

## Why pinned

The undated `nightly` channel moved to a compiler where the locked `branches`
dependency no longer builds (`core::intrinsics::abort` disappeared). Pinning
the date keeps Cargo, rustup, and Nix on the reviewed compiler and makes the
failure mode an explicit update decision instead of a moving target.

## Update workflow

1. Choose a new dated nightly and run at least:
   `nix develop -c rustc -Vv`,
   `nix develop -c cargo check --workspace --all-targets`,
   `nix develop -c cargo test`, and the focused gates for changed areas.
2. Check Casita's `rust-version`, the wasm/Apple targets in
   `rust-toolchain.toml`, and any component or Cargo-unit cohort that must
   remain distinct.
3. Update `rust-toolchain.toml`, the flake assertions/cohorts if needed, and
   this document in one reviewed change. Record the exact `rustc -Vv` output.
4. Do not change the compiler in the middle of a proof run.

A clean Cargo entry point and the checked-in Nix shell must report the same
version and commit; CI records `rustc -Vv` in the flake-check job.
