# Proposal: Recognize source-root musl tools in Rust provider scripts

## Summary

Teach Rust source provider bootstrap scripts to recognize the source-root musl toolchain layout (`x86_64-linux-musl-*`) as a valid implementation of the Rust musl target triple (`x86_64-unknown-linux-musl`).

## Motivation

The selectable musl-host route can validate, but the generated bootstrap scripts still prefer Nix wrapper-shaped `x86_64-unknown-linux-musl-*` tools and wrapper metadata. The source-built musl root available to Mantle exposes target-prefixed binaries as `x86_64-linux-musl-*` and sysroot files under `x86_64-linux-musl/`. Without recognizing that layout, the musl-host route remains blocked before a real provider can consume the source-root native closure.

## Scope

- Add deterministic tool-prefix/triple alias selection for the musl Rust target.
- Make generated first-stage and Rust-source scripts accept source-root musl tool layouts.
- Keep generic `cc`, `ld`, and `ld.lld` host-oriented; do not repurpose them as target tools.
- Preserve existing Nix-wrapper fallback behavior.

## Non-goals

- Do not claim a full source-built toolchain closure.
- Do not run or package the full Rust provider proof in this change.
- Do not change the default GNU-host route.
