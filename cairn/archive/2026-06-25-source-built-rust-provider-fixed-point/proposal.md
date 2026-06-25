# source-built Rust provider fixed-point

## Problem

Mantle now has a real source-built Rust provider and can materialize a zero-seed native toolchain closure manifest from the provider plus the source-root musl toolchain. The next claim is stronger than provider smoke: a Cargo-free Mantle build, and eventually a stage1/stage2 fixed point, must execute with that receipt-bound Rust/native closure rather than falling back to ambient host linker behavior.

The first attempted handoff exposes a boundary bug. Fixed-point compatibility probing currently happens before the receipt-bound PATH aliases are installed, so source-built `rustc` can fail to find the declared C compiler. Even after `cc` is available, the raw source-root musl GCC does not expose Rust's expected `-lunwind` archive name; the provider bootstrap solved that with a private target-linker runtime, but the broader Cargo-free proof path does not yet bind the same unwind archive in its explicit closure.

## Proposed change

Make the Cargo-free proof path use the explicit source-built toolchain closure for all Rust compiler compatibility probes and stage execution. Materialize the source-root GCC unwind archive as a receipt-bound runtime member, expose it through generated receipt-bound C compiler aliases, and fail closed if an explicit closure cannot satisfy the compatibility probe without an untracked wrapper.

Then run the real source-built provider handoff far enough to record either a successful zero-seed one-shot/fixed-point proof or the exact deterministic blocker that remains.

## Success criteria

- Native closure materialization records the source-root musl unwind archive with BLAKE3 digest and source/build receipt identity.
- Receipt-bound C compiler aliases make the declared unwind archive available as `libunwind.a` without relying on ambient PATH entries.
- Rust compatibility probes for explicit closures run under the receipt-bound PATH and fail closed instead of creating a host-shell/rustc wrapper that is outside the closure claim.
- Positive and negative tests cover receipt-bound probe PATH, unwind alias materialization, and explicit-closure probe failure.
- Evidence records the real provider-backed Cargo-free frontier and does not overclaim full release reproducibility, full Cargo compatibility, or a broader bootstrap proof.
