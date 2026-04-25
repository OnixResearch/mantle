## Why

crunch's bwrap sandbox has several gaps that allow host-dependent or
run-dependent information to leak into build outputs. Builds that embed
hostnames, read `/proc/cpuinfo`, consume `/dev/urandom`, or rely on
ambient file permissions can produce different outputs on different
machines or across runs. These gaps block bit-for-bit reproducibility
even when derivation inputs are identical.

Nix's sandbox addresses most of these vectors. crunch should match or
exceed that baseline.

## What Changes

- **Sandbox hostname**: set a fixed hostname inside the UTS namespace.
- **`/proc` masking**: hide host-specific hardware topology files.
- **`/dev` restriction**: replace full devtmpfs with a minimal device set.
- **Output permission normalization**: make exported file modes deterministic.
- **Output timestamp normalization**: reset mtimes on exported store paths.
- **`NIX_BUILD_CORES` default**: pin to a deterministic value.
- **Cgroup namespace isolation**: unshare cgroups to hide host cgroup paths.
- **`HashMap` audit in orchestrator**: replace non-deterministic iteration
  with ordered maps where iteration order can affect output.

## Capabilities

### New Capabilities
- `sandbox-hostname`: fixed hostname inside all sandbox builds
- `proc-masking`: hide `/proc/cpuinfo`, `/proc/meminfo`, and related files
- `dev-restriction`: minimal `/dev` without random/urandom
- `output-permission-normalization`: deterministic file modes on store outputs
- `output-timestamp-normalization`: deterministic mtimes on store outputs
- `cgroup-isolation`: unshared cgroup namespace

### Modified Capabilities
- `build-environment`: `NIX_BUILD_CORES` default changes from `0` to `1`
- `hermeticity-audit`: new audit kinds for permission and timestamp drift

## Impact

- **Files**: `vendor/snix-build/src/bwrap/mod.rs`,
  `crates/crunch-build/src/build_request.rs`,
  `crates/crunch-store/src/export.rs`,
  `crates/crunch-build/src/orchestrate.rs`,
  `crates/crunch-build/src/hermeticity.rs`
- **APIs**: `export_castore_to_disk()` gains permission/timestamp normalization;
  `SANDBOX_ENV_VARS` changes `NIX_BUILD_CORES` default
- **Dependencies**: none
- **Testing**: unit tests for each normalization path; self-hosting proof
  must still pass with the tightened sandbox
