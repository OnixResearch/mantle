# Spec: pure-shell-environment-core

## Summary

Shell environment computation lives in `crates/crunch-shell/` as a pure
function with no I/O, no subprocess exec, and no ambient state reads.

## Requirements

### Requirement: Pure activation function signature

`compute_activation()` MUST accept structured input (`ShellSidecar`,
`HostEnv`, `&[PathBuf]` for `--with` paths) and return a
`Result<ActivationPlan, ShellError>`. The function MUST NOT call
`std::env::var`, `std::fs::*`, `std::process::*`, `std::time::*`, or any
async runtime. The crate's `Cargo.toml` MUST NOT depend on `tokio`,
`crunch-build`, `crunch-store`, or any filesystem/network crate.

#### Scenario: Crate dependency audit

- GIVEN the `crates/crunch-shell/Cargo.toml` file
- WHEN its `[dependencies]` section is inspected
- THEN only `serde`, `serde_json`, and `thiserror` appear as non-dev
  dependencies

### Requirement: ActivationPlan carries all exec decisions as data

`ActivationPlan` MUST contain: an env var map (`BTreeMap<String, String>`),
an ordered PATH (`Vec<PathBuf>`), an optional hook string
(`Option<String>`), an `ExecTarget` enum, and a warnings list
(`Vec<ShellWarning>`). The plan MUST NOT hold file handles, process
handles, or references to ambient state. The env map MUST use `BTreeMap`
for deterministic iteration order.

#### Scenario: Plan is serializable round-trip

- GIVEN an `ActivationPlan` with env vars, PATH entries, a hook, and
  warnings
- WHEN serialized to JSON and deserialized back
- THEN the round-tripped plan equals the original

### Requirement: Protected variables are never overwritten

The core MUST maintain a `const` array of protected variable names:
`HOME`, `USER`, `TERM`, `LOGNAME`, `DISPLAY`, `LANG`, `SHELL`. If the
sidecar declares any of these, the core MUST skip them and record a
`ShellWarning::ProtectedVarSkipped { key }` in the plan.

#### Scenario: Sidecar declares HOME

- GIVEN a sidecar with `env.HOME = "/override"`
- AND a host env with `HOME = "/home/user"`
- WHEN `compute_activation()` runs
- THEN the plan's env map contains `HOME = "/home/user"`
- AND the plan's warnings contain `ProtectedVarSkipped { key: "HOME" }`

#### Scenario: Sidecar declares non-protected var

- GIVEN a sidecar with `env.RUST_LOG = "debug"`
- AND a host env without `RUST_LOG`
- WHEN `compute_activation()` runs
- THEN the plan's env map contains `RUST_LOG = "debug"`
- AND the plan's warnings are empty

### Requirement: Sidecar version is validated

The core MUST reject sidecars with `version` values other than `1` with
`ShellError::UnsupportedSidecarVersion { version }`. Missing `version`
is also an error.

#### Scenario: Version 2 sidecar

- GIVEN a sidecar with `version: 2`
- WHEN `compute_activation()` runs
- THEN it returns `Err(ShellError::UnsupportedSidecarVersion { version: 2 })`

### Requirement: Imperative shell has no logic

The imperative shell (`src/shell_cmd.rs`) MUST be: read sidecar from disk,
snapshot host env into `HostEnv`, call `compute_activation()`, match on
`ExecTarget`, call `std::process::Command`. No env merging, no PATH
dedup, no hook decision, no conditional logic beyond the `ExecTarget`
match.
