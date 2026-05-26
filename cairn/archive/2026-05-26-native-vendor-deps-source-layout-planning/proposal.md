# Proposal: Native vendor-deps source layout planning

## Summary

Support Mantle's conventional `vendor-deps/<crate>/Cargo.toml` registry source layout in native Rust planning. This removes a real Mantle self-build gap where vendored registry sources are present locally but not named with Cargo's `<name>-<version>` directory convention.

## Motivation

Mantle's checkout has a large local `vendor-deps/` tree. Native registry source planning must bind those sources by explicit local files and Cargo.lock checksums instead of falling back to `$CARGO_HOME`, a network index, or Cargo cache material.

## Scope

- Discover `vendor-deps/` as a declared local registry source root when present.
- Bind both `<name>-<version>` and `<name>` vendor directory layouts.
- Compare native registry package facts to Cargo oracle package IDs, not only Cargo's cache manifest paths.
- Preserve fail-closed checksum/source-root behavior.

## Non-goals

- Full Cargo feature resolution.
- Network/index access.
- Ambient Cargo registry cache fallback.
