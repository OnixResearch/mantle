# first-stage musl stage2 rustc probe runtime

## Problem

After source-root musl `libc.so` is made visible to `run_rustc`, a fresh real provider rerun advances through the previous `tracing_attributes` proc-macro frontier and then fails in `build-std2`. Cargo probes `rustc -vV` through `rustc_proxy.sh`, but the `CARGO_ENV_STAGE2_STD` Makefile variable does not include `RUSTC_ENV_VARS`, so the proxy launches the musl-host rustc without the private runtime search path. Cargo reports that `rustc -vV` had no `host:` line even though the same rustc returns a valid host line when run with the private runtime path.

## Proposed change

Extend the generated first-stage musl runtime normalization to rewrite `run_rustc/Makefile`'s `CARGO_ENV_STAGE2_STD` line so stage2 Cargo standard-library builds inherit `$(RUSTC_ENV_VARS)`. Keep the rewrite musl-route-only and fail closed if the expected Makefile line is absent. The change should preserve the private runtime scope and continue leaving generic host aliases host-oriented.

## Success criteria

- The generated first-stage script rewrites both the `LD_LIBRARY_PATH` line and the stage2 Cargo env line while accepting already-normalized lines for scratch idempotence.
- Focused `rust_source_provider` tests prove the stage2 env rewrite and existing runtime guards are present.
- Evidence records the fresh real `rustc -vV`/missing `host:` failure and scratch continuation showing the stage2 env rewrite advances to the next `__popcountdi2` linker frontier.
