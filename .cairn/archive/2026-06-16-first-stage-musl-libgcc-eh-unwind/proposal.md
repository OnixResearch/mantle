# first-stage musl libgcc_eh unwind binding

## Problem

The first-stage musl-host Rust provider now reaches the `run_rustc` hello-world link with the private source-root wrapper and non-PIE static mode. The next failure is unresolved `_Unwind_*` symbols because the wrapper exposes `-lunwind` as a copy of `libgcc.a`, while the source-root musl GCC stores the unwind personality and exception functions in `libgcc_eh.a`.

## Proposed change

Make the generated private first-stage musl target wrapper prefer `libgcc_eh.a` when it is present, copy that selected unwind archive to the private runtime directory as `libunwind.a`, and keep `libgcc.a` for libgcc-style aliases. If a toolchain folds unwind symbols into `libgcc.a`, the wrapper may fall back to that archive; otherwise the later link still fails closed with unresolved unwind symbols. This leaves generic host aliases unchanged and keeps the fix scoped to the generated first-stage process tree.

## Success criteria

- The generated first-stage wrapper prefers the target GCC CRT directory's `libgcc_eh.a` for `-lunwind` when that archive exists.
- `-lunwind` resolves to a private `libunwind.a` copied from the selected unwind archive.
- Focused tests, formatting, diff checks, and Cairn gates pass.
- Evidence records the real rerun frontier and source-root symbol probe proving `_Unwind_*` lives in `libgcc_eh.a`.
