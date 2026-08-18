# Design: Source-root musl tool recognition

## Problem

Mantle's route metadata can now select `host_triple = x86_64-unknown-linux-musl`, but generated provider scripts still assume the musl target compiler is named `x86_64-unknown-linux-musl-gcc` and that fallback toolchains carry Nix wrapper `nix-support/orig-libc` metadata. The source-root musl provider is not a Nix wrapper: it exposes `bin/x86_64-linux-musl-gcc`, companion `g++`/`ar`/`ranlib`, and a sysroot at `x86_64-linux-musl/`.

## Approach

Keep route-plan loading unchanged. Add a small pure mapping layer in `rust_source_provider.rs` that maps Mantle's Rust musl triple to accepted C tool prefixes and `gcc -dumpmachine` values:

- Rust triple: `x86_64-unknown-linux-musl`
- Accepted tool prefixes: `x86_64-unknown-linux-musl`, `x86_64-linux-musl`
- Accepted machine values: `x86_64-unknown-linux-musl`, `x86_64-linux-musl`
- Source-root sysroot suffix: `x86_64-linux-musl`

The script-generation shell then uses those constants instead of a single hard-coded tool name. For source-root layouts, sysroot detection falls back from Nix wrapper metadata to `$tool_root/$sysroot_suffix` and requires `lib/libc.a` plus CRT files before exporting the tool configuration.

## Safety and boundaries

Generic host aliases (`cc`, `ld`, `ld.lld`) stay untouched. The target linker wrapper only activates for the explicit musl target branch and still creates a private alias directory for `cc`, `c++`, `ar`, and `ranlib` scoped to the generated script.

## Validation

Focused validation will cover:

- pure mapping tests for accepted/rejected tool names and machine aliases,
- generated first-stage script contents for source-root aliases,
- generated Rust-source stage script contents for source-root sysroot fallback,
- existing default route tests to prove no GNU-host route regression.
