# Proposal: Distribute Rust compilation units

## Why

The existing Cargo unit-plan lane already separates Rust compilation into
ordinary derivations (`crates/mantle-cargo-unit-plan`,
`lib/cargo_unit_plan.ncl`). Its producer, source slices, helper, dependency
manifests and sandboxed build-script outputs are the right granularity for
distribution. Sending an opaque `offlineCargoPackage` root to one builder is
not equivalent to scheduling those units across a pool.

This change makes the existing lane a consumer of native pool dispatch. It
does not create a remote rustc protocol or promote host-side `rust-plan`
verification into the project build executor.

## What Changes

- Admit verified producer plan/source outputs and distribute ready ordinary
  units through the same generic Worker. r[distributed_compilation.rust_unit_graph]
- Bind compiler execution platform separately from artifact target, including
  proc-macro, build-script, host/target sysroot and linker requirements.
  r[distributed_compilation.rust_platform_roles]
- Transfer immutable complete source/tool/dependency closures, including
  generated files and typed build-script flags, before dependent compilation.
  r[distributed_compilation.rust_artifact_barriers]
- Preserve identical unit/action identities under placement changes and retain
  selective signed reuse without sharing incremental process state.
  r[distributed_compilation.rust_identity_reuse]
- Prove actual Rust compiler work overlaps on two workers, execute the resulting
  application, and retain lane-specific failure and non-claim evidence.
  r[distributed_compilation.rust_acceptance]

## Impact

- **Immediate consumer:** the supported Cargo unit-plan lane and the small
  workspace demonstrated by `examples/cargo_unit_plan.ncl`; extend the existing
  build-script/native-input fixture rather than claim the full Mantle graph.
- **Immediate outcome:** independent Rust units of one build run on different
  workers and their verified artifacts link into a working application.
- **Durable capability:** distributed finished-artifact execution of an
  existing frontend-neutral unit graph, with explicit host/target roles.
- **Maintenance owner:** Mantle Rust-planning owner for the producer, lowering,
  unit helper, Nickel interface and Rust compatibility matrix.
- **Repeatability evidence:** same-plan local/pool builds, real proc-macro and
  generated-source consumers, source/native-input perturbations, and independent
  clean stores with exact compiler/toolchain identities.
- **Compatibility:** the default offline Cargo lane and host-side Rust-plan
  verification remain separate. Pool mode is explicitly selected.

## Dependencies

`metadata.json` requires `schedule-native-goals-across-workers`,
`build-cargo-units-as-dynamic-plans` (already archived), and
`run-cargo-build-scripts-as-plan-units` (its remaining acceptance is not inferred
from source). Early-interface pipelining, warm workspaces and local compiler
caches are not prerequisites. See
[proposed ADR 0138](../../../adr/0138-distribute-compiler-units-through-native-build-goals.md).

## Non-Goals

Rust-specific generic scheduler/store branches; remote Cargo package resolution;
networked build scripts; shared `target/` or incremental directories; unfinished
metadata or persistent compiler sessions; transparent distribution of an opaque
Cargo derivation; expanded dynamic-plan bounds or broader Cargo compatibility.

## Success Criteria

A declared small graph with independent libraries, a real proc macro, build
script, generated input and final executable succeeds across two workers under
complete-artifact barriers. Placement changes preserve unit identities;
source/native-input edits invalidate the affected closure; compiler failure and
stale generated output cannot yield a signed successful application. Unsupported
graphs keep stable explicit blockers, with no Cargo compilation fallback.
