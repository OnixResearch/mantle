# Implementation validation: optional sandbox-shell compile default

Task-ID: I2,I3,V1,V2
Covers: r[rust_package_planning.source_built_toolchain_closure.snix_sandbox_shell_compile_env]
Date: 2026-06-26

## Implementation summary

Changed mandatory compile-time `env!("SNIX_BUILD_SANDBOX_SHELL")` reads to optional `option_env!("SNIX_BUILD_SANDBOX_SHELL")` with `/bin/sh` as the existing placeholder in:

- `vendor/snix-build/src/buildservice/bwrap.rs`
- `vendor/snix-build/src/buildservice/oci.rs`
- `src/operator_diagnostics.rs`

The bwrap runtime path still prefers an explicit non-placeholder runtime `SNIX_BUILD_SANDBOX_SHELL`, keeps existing non-placeholder compile defaults only when the path exists, and treats `/bin/sh` as a placeholder that may fall through to static busybox discovery or bounded fallback.

No source-built sandbox-shell claim was added. The change only removes the undeclared compile-time env blocker.

## V1: vendored snix-build compiles with env unset

Pueue task 25 ran:

```text
nix develop -c env -u SNIX_BUILD_SANDBOX_SHELL CARGO_TARGET_DIR=/tmp/mantle-snix-sandbox-shell-after cargo check -p snix-build --lib
```

Result:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 13.38s
```

## V2: Mantle binary compile path compiles with env unset

Pueue task 35 ran:

```text
nix develop -c env -u SNIX_BUILD_SANDBOX_SHELL CARGO_TARGET_DIR=/tmp/mantle-sandbox-shell-after-bin cargo check -p mantle --bin mantle
```

Result:

```text
warning: `mantle` (bin "mantle") generated 54 warnings (run `cargo fix --bin "mantle" -p mantle` to apply 1 suggestion)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 34.23s
```
