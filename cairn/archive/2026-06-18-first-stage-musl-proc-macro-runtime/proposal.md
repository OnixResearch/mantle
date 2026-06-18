# first-stage musl proc-macro runtime

## Problem

The real source-root musl-host Rust provider route now gets past the static CRT, unwind, and `rustc_driver` blockers, but `run_rustc` fails when the musl-built `tracing_attributes` proc macro is loaded by the compiler-host build. The proc macro is a shared object that needs source-root musl `libc.so`; without a private runtime search path, the host loader either cannot use the artifact or finds an incompatible ambient libc surface.

## Proposed change

Extend the generated first-stage script for the source-root musl compiler-host route so it copies source-root `libc.so` into the existing private target runtime directory and rewrites mrustc `run_rustc/Makefile`'s `RUSTC_ENV_VARS += LD_LIBRARY_PATH=$(abspath $(LIBDIR))` line to search that private runtime before the build libdir. Keep the change guarded to the musl host/target route, fail closed if the shared libc or expected Makefile line is missing, and continue leaving generic host aliases such as `cc`, `ld`, and `ld.lld` host-oriented.

Also scrub ambient `CARGO_BUILD_RUSTC_WRAPPER` so local sccache configuration cannot mask the provider frontier in scratch or real reruns.

## Success criteria

- The generated first-stage script contains a musl-only proc-macro runtime setup, copies `libc.so` into the private runtime dir, patches the exact mrustc `LD_LIBRARY_PATH` line, and fails closed on missing runtime or unexpected Makefile shape.
- Focused `rust_source_provider` tests prove the generated script includes the guarded runtime path and environment scrub markers without changing generic host aliases.
- Evidence records the real `tracing_attributes` failure and scratch continuation showing that adding source-root musl `libc.so` to `LD_LIBRARY_PATH` advances the frontier to the next compiler error.
