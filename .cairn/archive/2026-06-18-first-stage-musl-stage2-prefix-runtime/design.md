# Design: first-stage musl stage2 prefix runtime

## Context

The source-root musl first-stage script copies `libc.so` into a private runtime directory and rewrites `run_rustc/Makefile` so stage2 Cargo inherits `$(RUSTC_ENV_VARS)`. The inherited value still expands `LD_LIBRARY_PATH` as `$target_runtime_dir:$(abspath $(LIBDIR))`.

At the beginning of `build-std2`, Cargo probes `$(BINDIR_2)rustc` through `rustc_proxy.sh`. `$(BINDIR_2)rustc` is the stage2 rustc under `output/prefix-2/bin`, so its loader needs `output/prefix-2/lib` before the final `output/prefix/lib/rustlib/...` path exists or is relevant.

## Core behavior

The generated-script normalization for the existing `RUSTC_ENV_VARS += LD_LIBRARY_PATH=$(abspath $(LIBDIR))` line changes the replacement to:

`RUSTC_ENV_VARS += LD_LIBRARY_PATH=$target_runtime_dir:$(abspath $(PREFIX_2)lib):$(abspath $(LIBDIR))`

The existing pass remains idempotent and fail-closed:

1. Original upstream line is rewritten once.
2. Already-normalized line is accepted for scratch reruns.
3. Missing original and normalized shapes abort before invoking `run_rustc`.

This keeps the runtime path private to the generated first-stage script. It does not replace generic `cc`, `ld`, `ld.lld`, Cargo, or host linker aliases.

## Validation

Focused tests inspect generated script text for the private source-root runtime, `$(PREFIX_2)lib`, stage2 `$(RUSTC_ENV_VARS)` inheritance, and fail-closed diagnostics. Scratch evidence reruns the failed preserved `run_rustc` stage with the Makefile line patched and records the next frontier.
