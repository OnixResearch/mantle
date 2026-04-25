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
- **`/dev` restriction**: keep bwrap's device baseline but mask entropy devices and isolate `/dev/shm`.
- **`/sys` masking**: hide sysfs hardware topology and device information.
- **FOD `/etc` isolation**: use synthetic `/etc/resolv.conf` and `/etc/services`
  for network-enabled builds instead of bind-mounting host files.
- **Output permission normalization**: make exported file modes deterministic.
- **Output timestamp normalization**: reset mtimes on exported store paths
  (files, directories, and symlinks).
- **`NIX_BUILD_CORES` default**: pin to a deterministic value.
- **Cgroup namespace isolation**: request best-effort cgroup namespace unsharing to hide host cgroup paths when kernel support is available.
- **`HashMap` audit in orchestrator**: replace non-deterministic iteration
  with ordered maps where iteration order can affect output.

## Capabilities

### New Capabilities
- `sandbox-hostname`: fixed hostname inside all sandbox builds
- `proc-masking`: hide `/proc/cpuinfo`, `/proc/meminfo`, and related files
- `sys-masking`: hide `/sys/devices/system/cpu/` and related sysfs paths
- `dev-restriction`: bwrap-managed `/dev` with `/dev/random` and `/dev/urandom` masked and private `/dev/shm`
- `fod-etc-isolation`: synthetic resolv.conf/services for FOD builds
- `output-permission-normalization`: deterministic file modes on store outputs
- `output-timestamp-normalization`: deterministic mtimes on store outputs
  (including symlink lmtime)
- `cgroup-isolation`: unshared cgroup namespace

### Modified Capabilities
- `build-environment`: `NIX_BUILD_CORES` default changes from `0` to `1`

## Impact

- **Files**: `vendor/snix-build/src/bwrap/mod.rs`,
  `crates/crunch-build/src/build_request.rs`,
  `crates/crunch-store/src/export.rs`,
  `crates/crunch-build/src/orchestrate.rs`
- **APIs**: `export_castore_to_disk()` gains permission/timestamp normalization;
  `SANDBOX_ENV_VARS` changes `NIX_BUILD_CORES` default
- **Dependencies**: `filetime` crate added to `crunch-store`
- **Testing**: unit tests for each normalization path; self-hosting proof
  must still pass with the tightened sandbox

## Deferred / Future Work

- **Seccomp filtering**: syscall allowlisting for sandbox processes (deeper
  defense, separate effort with its own compatibility surface)
- **Time namespace (`--unshare-time`)**: kernel 5.6+ feature that could
  prevent `clock_gettime(CLOCK_REALTIME)` leakage, but bwrap does not
  support it yet
- **Full `/proc` virtualization**: current approach masks individual files;
  a procfs-filtering FUSE layer would be more complete but much heavier
- **Permission/timestamp drift audit kinds**: export-time normalization is
  deterministic; explicit drift detection/auditing is left to a later policy
  change if operators need to report pre-normalized metadata
