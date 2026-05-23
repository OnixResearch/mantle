# Change: execute-rust-build-script-metadata

## Problem

Mantle can compile explicit Rust host units and bind host artifacts into target units, but `custom-build` units are not yet run to produce bounded build-script metadata. Target units therefore cannot consume deterministic `OUT_DIR`, `cargo:rustc-cfg`, or `cargo:rustc-env` evidence without Cargo orchestration.

## Scope

Add a bounded build-script metadata execution rail for explicit `custom-build` host units already present in `unit_derivation_graph`.

In scope:
- Execute compiled build-script host artifacts with deterministic minimal environment.
- Capture and parse supported `cargo:` lines: `rustc-cfg`, `rustc-env`, `rerun-if-changed`, `rustc-link-lib`, and `rustc-link-search`.
- Bind generated metadata into supported target `lib`/`bin` consumers before invoking target `rustc`.
- Emit deterministic host-artifact topology JSON receipt evidence.
- Fail closed when generated metadata material is missing or malformed.

Out of scope:
- Full Cargo build-script compatibility.
- Native link probing or arbitrary host environment discovery.
- Network, registry, or Cargo target-dir execution.
- Test/doctest/example units.
