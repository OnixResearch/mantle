# first-stage musl stage2 prefix runtime

## Problem

The stage2 Cargo probe fix made `CARGO_ENV_STAGE2_STD` inherit `$(RUSTC_ENV_VARS)`, but the inherited `LD_LIBRARY_PATH` still points at the final `$(LIBDIR)` path. During the stage2 standard-library build, Cargo probes `output/prefix-2/bin/rustc` through `rustc_proxy.sh`; that binary needs the adjacent `output/prefix-2/lib` runtime directory, so a fresh committed-code provider rerun still fails with Cargo reporting that `rustc -vV` had no `host:` line.

## Proposed change

Extend the private source-root musl runtime normalization so `RUSTC_ENV_VARS` prepends both the copied source-root musl runtime directory and `$(PREFIX_2)lib` before `$(LIBDIR)`. Keep the normalization route-local to the source-root musl first-stage script, preserve fail-closed Makefile shape checks, and continue avoiding global replacement of host aliases.

## Success criteria

- The generated first-stage script rewrites `RUSTC_ENV_VARS += LD_LIBRARY_PATH=$(abspath $(LIBDIR))` to include `$target_runtime_dir`, `$(PREFIX_2)lib`, and `$(LIBDIR)` in that order.
- Focused tests prove the generated script contains the `$(PREFIX_2)lib` runtime path and existing stage2 env inheritance.
- Evidence records the committed-code missing-`host:` rerun and a preserved-scratch continuation showing the `prefix-2` runtime path moves back to the next linker frontier.
