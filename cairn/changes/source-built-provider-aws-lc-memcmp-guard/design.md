## Context

The provider-bound release evidence run attempted to regenerate a current-code provider-backed fixed point. Stage1 planned successfully but topology execution stopped after 11 units because the `aws-lc-sys` build script ran `builder/cc_builder.rs::memcmp_check()` and the receipt-bound `cc` compiler produced a failing probe binary. The build script reports this as GCC bug PR95189 and recommends against using that compiler.

The active source-built native closure manifest binds `cc` and target aliases to the source-root musl GCC toolchain. The closure contains no independent receipt-bound Clang/C compiler alternative. Therefore a quick bypass such as changing `HOST`/`TARGET`, suppressing the guard, or falling back to `/usr/bin/cc` would overclaim the source-built toolchain closure.

## Decisions

### 1. Treat the guard as compiler-risk evidence, not a nuisance check

**Choice:** Mantle must either route AWS-LC C compilation to a receipt-bound compiler that passes the guard, or fail closed with a deterministic compiler-guard blocker.

**Rationale:** AWS-LC's guard is designed to detect a known unsafe compiler behavior. Ignoring it would make the provider proof less honest than upstream's build script.

### 2. Keep route selection in pure planning data

**Choice:** Any compiler selection or per-package policy should be derived from explicit package/unit/toolchain facts and recorded in receipts. Process execution remains in the imperative shell.

**Rationale:** This preserves Mantle's functional-core/imperative-shell split and lets tests exercise routing decisions without launching the full provider proof.

### 3. Ban ambient env spoofing as a success path

**Choice:** Do not satisfy the proof by setting `HOST != TARGET`, forwarding arbitrary `CC`/`CFLAGS`, or allowing an undeclared host compiler. Negative tests should cover these escapes.

**Rationale:** Those paths would skip or bypass the guard without proving the selected compiler is safe and source-built.

## Risks / Trade-offs

- A fully safe route may require extending the source-built native closure with another source-built C compiler or repairing the existing GCC route; that could be larger than a narrow Rust planner patch.
- If no safe source-built compiler is available yet, the honest outcome is a deterministic blocker rather than a current provider-bound release artifact.
