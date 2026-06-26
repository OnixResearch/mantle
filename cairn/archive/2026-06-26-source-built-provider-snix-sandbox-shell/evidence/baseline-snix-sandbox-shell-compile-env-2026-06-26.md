# Baseline: snix-build compile-time sandbox shell blocker

Task-ID: I1
Covers: r[rust_package_planning.source_built_toolchain_closure.snix_sandbox_shell_compile_env]
Date: 2026-06-26

## Command

Pueue task 18 ran:

```text
nix develop -c env -u SNIX_BUILD_SANDBOX_SHELL CARGO_TARGET_DIR=/tmp/mantle-snix-sandbox-shell-baseline cargo check -p snix-build --lib
```

## Result

The baseline failed before implementation because vendored `snix-build` required an undeclared compile-time environment variable:

```text
error: environment variable `SNIX_BUILD_SANDBOX_SHELL` not defined at compile time
  --> vendor/snix-build/src/buildservice/oci.rs:32:37
   |
32 | ...T: &str = env!("SNIX_BUILD_SANDBOX_SHELL");
   |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: use `std::env::var("SNIX_BUILD_SANDBOX_SHELL")` to read the variable at run time

error: environment variable `SNIX_BUILD_SANDBOX_SHELL` not defined at compile time
  --> vendor/snix-build/src/buildservice/bwrap.rs:33:37
   |
33 | ...T: &str = env!("SNIX_BUILD_SANDBOX_SHELL");
   |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: use `std::env::var("SNIX_BUILD_SANDBOX_SHELL")` to read the variable at run time

error: could not compile `snix-build` (lib) due to 2 previous errors
```

## Decision

The blocker is a compile-time ambient-env dependency, not evidence that a source-built sandbox shell was missing from the native closure. The fix should remove mandatory `env!` while preserving bounded runtime-shell non-claims.
