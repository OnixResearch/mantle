# first-stage musl static CRT normalization

## Problem

The musl-host Rust source provider now rewrites first-stage `-static-pie` links to `-static`, but a real rerun showed the produced static musl `hello_world` and `compiler_builtins` build-script binaries still segfault before Rust `main`. The wrapper preserves Rust's `rcrt1.o` startup object while changing the link mode to non-PIE static, so musl starts with a PIE CRT object under a non-PIE static link.

## Proposed change

Copy `crt1.o` into the private first-stage target runtime directory and, when the wrapper normalizes `-static-pie` to `-static`, rewrite the accompanying `rcrt1.o` startup argument to `crt1.o`. Keep this private to the generated source-root musl target wrapper so generic host aliases remain host-oriented.

## Success criteria

- The generated wrapper has an explicit two-pass mapping that detects `-static-pie`, passes `-static`, and maps `rcrt1.o` to private `crt1.o` only for normalized static links.
- Focused tests prove the script contains `crt1.o`, the static-PIE detection flag, and the conditional startup-object mapping.
- Evidence records the real rerun failure: static musl `hello_world` and `compiler_builtins` build-script binaries segfault in `_start_c` before Rust code runs.
