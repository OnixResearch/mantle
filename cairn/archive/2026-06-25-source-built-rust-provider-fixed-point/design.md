# Design: source-built Rust provider fixed point

## Context

The accepted source-built closure requirements distinguish three states:

1. A validated Rust provider can retire the stale Rust-provider non-claim for provider-only proof slices.
2. An explicit toolchain closure manifest is authoritative when supplied.
3. Only enforced stage execution with a zero-seed explicit closure may claim the broader source-built toolchain closure for Cargo-free builds or fixed-point proofs.

The materialized source-root musl closure already declares `rustc`, sysroot, C compiler, linker, CRT, libc/libgcc runtime, and target-prefixed helpers. The first real source-built provider handoff shows two missing pieces in the shell boundary rather than in the pure manifest validator: compatibility probes need the same receipt-bound PATH as later stages, and the source-root GCC unwind archive must be exposed under Rust's expected `libunwind.a` linker name.

## Decisions

### 1. Keep the closure manifest authoritative

**Choice:** Add the unwind archive as another source-built runtime member in the explicit native closure manifest.

**Rationale:** The proof path should not synthesize untracked runtime inputs. The source-root archive is already part of the provider root; recording its BLAKE3 digest and provider identity makes the later PATH adapter reviewable.

### 2. Treat PATH aliases as proof adapters, not closure members

**Choice:** Generate a receipt-bound `cc` alias that copies the declared unwind archive into an alias-local runtime directory as `libunwind.a` and invokes the declared compiler with `-L<runtime-dir>`.

**Rationale:** The alias is an adapter that makes a declared source-built runtime member usable by Rust's linker convention. The closure claim remains bound to the declared archive digest, while the generated adapter stays inside the proof bundle and PATH remains isolated to the receipt-bound directory.

### 3. Fail closed for explicit-closure compatibility probes

**Choice:** Run `rustc -C link-self-contained=no` probes with the receipt-bound PATH when a toolchain closure is supplied. If that probe fails, return a closure blocker instead of generating the legacy compatibility wrapper.

**Rationale:** The legacy wrapper is useful for host Rust compatibility, but it is not a source-built toolchain member. Allowing it under an explicit closure would weaken the claim and could hide a missing source-built linker/runtime input.

## Validation

Focused validation covers pure closure materialization tests, Cargo-free proof shell tests, rustfmt, diff checks, and Cairn gates. Real evidence should then materialize a zero-seed closure from the rerun38 provider and source-root musl toolchain, run a provider-backed Cargo-free one-shot, and if that succeeds launch the full fixed-point proof.

## Risks / trade-offs

- The alias-local `libunwind.a` copy is generated proof adapter state, not a new source-built provider output. Tests must keep it tied to the declared archive bytes.
- The full fixed-point proof can still expose downstream native topology blockers unrelated to provider materialization; those blockers should be recorded without overclaiming success.
- Existing host-Rust fixed-point behavior must keep the legacy compatibility wrapper path when no explicit closure is supplied.
