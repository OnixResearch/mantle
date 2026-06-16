# Design: first-stage musl static-pie normalization

## Boundary

The repair is intentionally scoped to `src/rust_source_provider.rs` first-stage script generation. It changes only the private `$BUILD_DIR/target-linker-bin/cc` wrapper that is created when the musl-host route uses a source-root target GCC for `x86_64-unknown-linux-musl` links.

## Functional core

The pure behavior is script text generation: when the wrapper maps incoming linker arguments, it recognizes `-static-pie` and emits `-static` instead. Existing CRT path mapping and LFS compatibility object injection remain unchanged.

## Imperative shell

The generated shell wrapper still delegates to `$target_cc_path` and stays process-local to the first-stage script. The outer materializer continues to fail closed if the first-stage build fails.

## Validation

Focused validation runs rustfmt, the script-generation/materializer tests that cover source-root musl wrapper behavior, `git diff --check`, and Cairn gates. Frontier evidence records the long rerun and the small direct link probe showing why this normalization is necessary for the current source-root musl seed.
