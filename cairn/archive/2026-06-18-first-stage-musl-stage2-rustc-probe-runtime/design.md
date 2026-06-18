# Design: first-stage musl stage2 rustc probe runtime

## Context

The first-stage musl route now copies source-root `libc.so` into the private target runtime directory and patches `RUSTC_ENV_VARS` to prepend that directory to `LD_LIBRARY_PATH`. That environment is used by later `run_rustc` compiler-host Cargo builds, but stage2 standard-library builds use a separate `CARGO_ENV_STAGE2_STD` variable that launches Cargo and `rustc_proxy.sh` without `RUSTC_ENV_VARS`.

Cargo probes `rustc -vV` before compiling stage2 std. Without the private runtime path, the proxy can produce no usable verbose-version output, causing Cargo to report that the output lacks `host:`.

## Core behavior

The generated-script text normalization extends the existing Makefile pass:

1. Define the exact original stage2 line:
   `CARGO_ENV_STAGE2_STD := CARGO_TARGET_DIR=$(OUTDIR)build-std2 RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_S)rustc) $(CARGO_ENV)`.
2. Rewrite it to prepend `$(RUSTC_ENV_VARS)`.
3. Accept the already-normalized line for idempotent scratch reruns.
4. Fail closed when neither shape appears.

The imperative shell remains the generated first-stage script; no global host alias changes are needed.

## Validation

Focused tests inspect the generated script for both the original and normalized stage2 line, plus the existing runtime guard. Fresh evidence compares the real missing-`host:` failure to a scratch continuation where the stage2 rewrite advances to a later `__popcountdi2` link frontier.
