## Why

The current-code provider-backed fixed-point proof moved past AWS-LC and now blocks while compiling vendored `snix-build`: `env!("SNIX_BUILD_SANDBOX_SHELL")` requires an ambient compile-time environment variable that is not part of Mantle's source-built native closure receipt. That makes the proof depend on undeclared caller state before it can reach the next real topology frontier.

Mantle should compile the build-service code without an ambient sandbox-shell env while keeping the claim bounded: absence of a compile-time shell becomes the existing `/bin/sh` placeholder, not a source-built sandbox-shell claim. Runtime sandbox execution still uses explicit runtime configuration or existing runtime discovery/failure behavior.

## What Changes

- Add a requirement for provider-backed Cargo-free topology execution to tolerate missing compile-time `SNIX_BUILD_SANDBOX_SHELL` without importing ambient host state into proof claims.
- Replace mandatory `env!("SNIX_BUILD_SANDBOX_SHELL")` uses in Mantle/snix build-service code with `option_env!` plus the existing `/bin/sh` placeholder.
- Validate both vendored `snix-build` and the Mantle binary compile with `SNIX_BUILD_SANDBOX_SHELL` unset.
- Rerun the real provider-backed fixed-point proof and record success or the next deterministic blocker.

## Impact

- **Files**: `vendor/snix-build/src/buildservice/{bwrap,oci}.rs`, `src/operator_diagnostics.rs`, Cairn specs/evidence.
- **Testing**: baseline unset-env compile failure, post-change unset-env compile checks, focused formatting/tests, Cairn validation/gates, and a real provider-backed fixed-point rerun.
