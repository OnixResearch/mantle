# Design: first-stage musl proc-macro runtime

## Context

The source-root musl route builds compiler-host artifacts for `x86_64-unknown-linux-musl`. After the `rustc_driver` crate-type normalization, the real rerun reaches proc-macro loading: `libtracing_attributes-*.so` is built, but it needs the source-root musl dynamic runtime at load time.

The existing first-stage target wrapper already owns a private runtime directory for CRT objects and libgcc/unwind archives. Reusing that directory keeps the runtime visible only inside the generated provider script and avoids replacing generic host aliases.

## Core behavior

The pure generated-script core is deterministic text generation:

1. Guard the normalization to `RUSTC_TARGET == x86_64-unknown-linux-musl` and `RUSTC_PROVIDER_TARGET_TRIPLE == x86_64-unknown-linux-musl`.
2. Copy `$target_musl_crt_dir/libc.so` into `$target_runtime_dir/libc.so`.
3. Rewrite exactly `RUSTC_ENV_VARS += LD_LIBRARY_PATH=$(abspath $(LIBDIR))` in `run_rustc/Makefile` to search `$target_runtime_dir` first.
4. Accept an already-normalized line for idempotent scratch reruns.
5. Fail closed if the source-root shared libc, Makefile, or expected line is absent.

The imperative shell remains the generated script that runs in the materializer scratch directory.

## Validation

Focused tests inspect the generated script for the guard, copied runtime, exact Makefile line, idempotence/fail-closed state, and ambient `CARGO_BUILD_RUSTC_WRAPPER` scrub. Scratch evidence records that the private runtime path advances from the `tracing_attributes` dynamic load blocker to the next compiler error.
