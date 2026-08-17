# Design: first-stage musl libgcc_eh unwind binding

## Boundary

The change is limited to first-stage Rust source provider script generation in `src/rust_source_provider.rs`. It only affects the private `$BUILD_DIR/target-linker-bin/cc` wrapper used for musl target links during the mrustc `run_rustc` stage.

## Functional core

The generated script text must:

1. Require `libgcc.a` in the selected target GCC CRT directory.
2. Prefer `libgcc_eh.a` as the selected unwind archive when it exists; otherwise fall back to `libgcc.a` for target toolchains that fold unwind symbols there.
3. Copy the selected unwind archive to `$target_runtime_dir/libunwind.a` so Rust's `-lunwind` request resolves through the private runtime directory.
4. Keep `libgcc.a` copied for libgcc-style aliases.

## Imperative shell

The materializer continues to emit and execute the generated script, and it still fails closed if the script exits non-zero. No ambient `cc`, `ld`, or global PATH mutation is introduced.

## Validation

Focused validation covers script-generation assertions, musl-host materializer metadata tests, formatting, whitespace checks, and Cairn validation/gates. Evidence includes the failed rerun log and a direct source-root `nm` probe that distinguishes `libgcc.a` from `libgcc_eh.a` for the active source-root seed.
