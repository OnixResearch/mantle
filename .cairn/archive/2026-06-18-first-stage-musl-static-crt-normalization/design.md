# Design: first-stage musl static CRT normalization

## Context

The first-stage musl wrapper is a generated private `cc` script under `$BUILD_DIR/target-linker-bin`. It already maps source-root CRT paths into `$BUILD_DIR/target-linker-runtime`, rewrites unsupported `-static-pie` to `-static`, and injects compatibility/runtime archives for the Rust `run_rustc` target links.

A real musl-host rerun progressed past the former link failures, but the resulting static binaries segfault in musl `_start_c`. That is consistent with retaining `rcrt1.o` (static PIE startup) while replacing the link mode with non-PIE `-static`.

## Decisions

### 1. Track static-PIE normalization before mapping startup objects

**Choice:** The generated wrapper records whether any original argument was `-static-pie` before rewriting arguments.

**Rationale:** The startup-object decision depends on the original link request, not on the already-normalized argument stream. A small explicit flag keeps the wrapper deterministic and reviewable.

### 2. Copy both musl startup object variants into the private runtime dir

**Choice:** The wrapper setup copies `crt1.o`, `rcrt1.o`, `crti.o`, and `crtn.o` from the selected source-root musl sysroot.

**Rationale:** Keeping both startup objects local preserves the existing PIE-capable path while allowing the normalized non-PIE static path to use the matching `crt1.o` object.

### 3. Rewrite `rcrt1.o` to `crt1.o` only when static PIE is normalized

**Choice:** The generated wrapper maps `rcrt1.o` to private `crt1.o` only when the static-PIE flag was observed; otherwise it preserves the existing private `rcrt1.o` mapping.

**Rationale:** The fix is targeted to the known broken combination and does not silently change other future PIE-capable uses of `rcrt1.o`.

## Risks / Trade-offs

- This remains a first-stage musl wrapper workaround for the current source-root seed libc. It does not claim full source-built Rust provider success until the real rerun completes and downstream receipt-bound closure evidence reports `claim = true`.
- The script generation stays shell-based because it is part of the existing first-stage materializer shell. Tests assert exact generated fragments to keep the behavior deterministic.
