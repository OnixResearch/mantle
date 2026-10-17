# Proposal: Pipeline Rust interface readiness

## Why

Rust unit dependencies currently name finished artifacts in Mantle's native
paths. A dependent's analysis can sometimes start earlier than its code
generation or linking. Headstart demonstrates that distinction by emitting
early interface metadata and later replacing it with full metadata, while
leaving producer failure authoritative.

This is compiler-assisted work, not a generic scheduler preference. The
reviewed Headstart revision
[`ff81bfecda2116d54776a7fec538fc2f4e349d1f`](https://github.com/PowderworksCode/headstart/tree/ff81bfecda2116d54776a7fec538fc2f4e349d1f)
contains an unfixed incremental SVH identity panic and a local colocated-file
waiting protocol. It is not an admitted default toolchain or evidence of a
Mantle speedup. Reviewed source facts are dated 2026-10-15.

## What Changes

- Require an explicit compiler/Cargo capability and identity gate, including a
  regression-backed fix for the early/full metadata incremental identity bug.
  Do not count disabling incremental compilation or changing the metadata hash
  scheme as resolving that prerequisite.
  r[rust_package_planning.phase_readiness.compiler_gate]
- First qualify a deliberately opt-in patched Cargo/rustc session inside one
  existing offline Cargo sandbox. Preserve whole-derivation output admission,
  declared inputs, offline behavior, and bounded session policy.
  r[rust_package_planning.phase_readiness.sandbox_trial]
- Represent interface-ready, full-metadata-ready, and link-artifact-ready
  requirements in the Rust-owned pure model, with exact producer attempts and
  conservative final-artifact edges for proc macros, dylibs, and script runs.
  r[rust_package_planning.phase_readiness.typed_requirements]
- Extend the Rust application executor with bounded live notifications and
  staged dependency eligibility, reusing provisional confirmation/teardown.
  Compare ordinary full-metadata pipelining before claiming an early-interface
  benefit.
  r[rust_package_planning.phase_readiness.execution]
- Gate acceptance on real differential and resource evidence for declared
  supported topologies; keep default Cargo behavior and ordinary per-unit
  store-output semantics unchanged.
  r[rust_package_planning.phase_readiness.acceptance]

## Dependencies

Metadata declares `explain-rust-build-waits`,
`validate-rust-pipelining-equivalence`, and
`bound-provisional-rust-execution`. These independently deliver observation,
comparison, and admission mechanisms before this feature is enabled.

External prerequisite: admit a compatible compiler revision containing an
actual fix for Headstart's incremental-identity defect and the supported
phase/locking semantics. The reviewed upstream bases are rustc
`6006fd03947bd7b04ae5cb5f7f9bb66b44dc8c6e` and Cargo
`4f3fb2428282fb2467c4bb5be0163d2f42a31491`; they are provenance, not a claim
that the current Mantle toolchain accepts the patches. An unresolved gate
blocks execution, not authoring these proposals or landing the independent
measurement and validation changes.

## Impact

- **Immediate consumer:** admitted small Rust dependency-chain builds whose
  timeline demonstrates idle dependency-wait capacity.
- **Immediate outcome:** a bounded optional session overlaps dependent analysis
  without accepting failed producers or publishing speculative outputs.
- **Durable capability:** Rust-owned readiness requirements and session
  execution independent of generic scheduler/store completion semantics.
- **Maintenance owner:** Mantle Rust planning/execution maintainers; the optional
  provider owner maintains its exact upstream/patch/runtime inputs.
- **Affected files:** `lib/offline_cargo.ncl` and its evidence adapter,
  `crates/mantle-rust-plan-core/src/plan.rs`,
  `crates/mantle-rust-plan-app/src/application.rs`, `src/rust_plan.rs`, and
  compatible compiler wrapper/session adapters.
- **Repeatability evidence:** pinned same-toolchain off/on runs, serial and
  full-metadata baselines, real stage overlap, failure/incremental parity,
  memory/process bounds, and final cache visibility.
- **Compatibility:** no default mode switch, no new cross-sandbox live-file
  capability, and no host `rust-plan` output promotion into ordinary PathInfo.

## Non-Goals

- Fixing rustc itself in Mantle, adopting a rolling compiler fork, or promising
  upstream merge readiness.
- Publishing `.early-rmeta` as an ordinary completed derivation output, changing
  generic `GoalState`, or transferring live locks/files to remote builders.
- Replacing Cargo's existing normal metadata pipelining or the generic Mantle
  priority scheduler.
- Expanding supported Cargo semantics, lifting source-slice limits to make the
  full Mantle workspace fit, or claiming self-hosting/bootstrap/release proof.

## Success Criteria

The fixed and admitted compiler/provider passes unchanged warm incremental,
body/interface edit, break/fix, edit/revert, and clean-vs-incremental identity
checks under the normal hash scheme. An opt-in one-sandbox trial preserves
existing output authority. A supported native session starts actual dependent
analysis on the declared readiness phase, gates code generation/linking
correctly, confirms only successful producer closures, and terminates within
all declared budgets. Missing capability remains an explicit blocked result.
A pilot alone does not complete the native-executor requirements; all tasks
remain unchecked until their own evidence exists.
