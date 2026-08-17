## Context

`execute_rust_unit()` and `run_build_script_metadata()` intentionally call `env_clear()` before invoking child processes. That keeps topology execution deterministic, but it also strips `PATH`. Some toolchain components are absolute paths yet delegate through scripts with `/usr/bin/env bash`; without `PATH`, these scripts fail before the topology rail reaches the next modeled Rust dependency issue.

## Decision

### 1. Preserve only invocation PATH as tool PATH

Use a small functional core that merges explicit derivation environment with an optional inherited `PATH`. The imperative shell applies the merged map after `env_clear()`.

### 2. Keep derivation env explicit

Derivation-provided keys remain explicit and override the inherited `PATH` if a future derivation deliberately supplies one. Existing derivations do not supply `PATH`, so the caller's tool PATH becomes available only to child tool wrappers.

### 3. Test the merge logic directly

Focused tests cover non-empty inherited PATH propagation, empty inherited PATH omission, and derivation PATH override. This avoids rustc/link execution flakiness while proving the environment construction seam.

## Risks

- Passing caller PATH to child processes is less hermetic than an empty PATH. This is accepted for the topology execution rail because rustc/linker paths are already explicit, and PATH is needed for script interpreters in the selected toolchain.
