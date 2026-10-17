# Design: Pipeline Rust interface readiness

## Current behavior and fit

`lib/offline_cargo.ncl` runs declared `cargo build --locked --offline` inside one
sandbox. Cargo owns its internal scheduling and already supports ordinary full
metadata pipelining. Mantle accepts the package derivation only at completion.
This is the smallest boundary for a first compiler-assisted experiment.

The host `rust-plan` executor's `execute_existing_units` loop in
`crates/mantle-rust-plan-app/src/application.rs:125-181` is serial and finishes
each unit before visiting dependents. Its normalized effects and exact process
identity types live in `crates/mantle-rust-plan-core/src/plan.rs`. This separate
host-side lane has execution evidence, not ordinary derivation authority.

The opt-in `mantle-cargo-unit-plan` lane emits ordinary per-unit sandbox
outputs. `unit_helper.rs:80-97,486-504` consumes finished linkable libraries and
writes manifests after rustc exits successfully. Adding a rustc flag does not
change those completed-output edges. Current operator documentation also
excludes Mantle's 711-source workspace from the 256-source-slice bounded lane;
this proposal does not lift that unrelated bound.

## Reviewed reference and external blocker

The reviewed source is Headstart revision
[`ff81bfecda2116d54776a7fec538fc2f4e349d1f`](https://github.com/PowderworksCode/headstart/tree/ff81bfecda2116d54776a7fec538fc2f4e349d1f),
reviewed 2026-10-15. Relevant pinned files:

- [Design and phase/lock protocol](https://github.com/PowderworksCode/headstart/blob/ff81bfecda2116d54776a7fec538fc2f4e349d1f/docs/design.md).
- [Known incremental SVH panic](https://github.com/PowderworksCode/headstart/blob/ff81bfecda2116d54776a7fec538fc2f4e349d1f/docs/ice-early-metadata-svh.md).
- [Readiness, memory, jobserver, and platform gaps](https://github.com/PowderworksCode/headstart/blob/ff81bfecda2116d54776a7fec538fc2f4e349d1f/docs/readiness.md).
- [Six rustc and three Cargo patches and exact bases](https://github.com/PowderworksCode/headstart/blob/ff81bfecda2116d54776a7fec538fc2f4e349d1f/patches/README.md).

The recorded bug occurs when early metadata is re-encoded in another order
while full metadata and its SVH are reused from an incremental session. The
external gate requires a source fix plus the affected unchanged-warm and
edit/revert regression, using ordinary metadata identity. Disabling incremental
compilation, flushing the cache, ignoring the panic, or setting
`-Zmetadata-crate-hash=no` is not prerequisite completion. Do not rerun a known
failing upstream scenario merely to rediscover it during proposal authoring.

## Decisions

### Decision: Capability admission precedes every experimental execution

**Choice:** Admit exact rustc/Cargo bytes and revisions, reviewed patch/source
identities, sysroot/runtime closure, supported phase protocol, policy, and
semantic flags through declared inputs. The optional provider must demonstrate
that its fixed incremental identity and metadata-swap semantics work for the
required fixture set. Bind `-Zearly-metadata`, verify settings, and Cargo's
Headstart selection into admitted arguments/config, not ambient `RUSTFLAGS` or
a globally exported unstable environment variable.

**Rationale:** A binary named rustc or a nightly version is not a capability
proof. Known bases are rustc `6006fd03947bd7b04ae5cb5f7f9bb66b44dc8c6e` and
Cargo `4f3fb2428282fb2467c4bb5be0163d2f42a31491`; a later pin must record its
actual provenance and regression evidence. Review reuse terms before importing
external patches or scripts; public source visibility alone is not permission.
No compiler/source import occurs while authoring this package.

### Decision: One sandbox first, native application second

**Choice:** Stage A runs patched Cargo and rustc inside one existing offline
Cargo package sandbox, with the prior whole-package publication boundary.
Require the bounded provisional-session policy and compatible provider
behavior; the known upstream token-free resume cannot be silently accepted as
strict reservation enforcement. If the provider cannot enforce the selected
policy, the trial is blocked.

Retain off/on raw diagnostics, status, output observations, CPU and process-tree
memory, resident/paused counts, error latency, and teardown. Use identical
binaries, workload inputs, resource policy, and cache cohorts. Stage B extends
the existing Rust application/core executor for supported native topologies
only after Stage A safety evidence and wait diagnostics identify a concrete
critical-chain opportunity. Both stages are part of this proposal; passing the
trial alone is not native feature completion.

**Rationale:** The initial experiment need not weaken immutable store inputs or
add a capability for one sandbox to observe another's mutable output. The
host-side executor retains its existing evidence-only authority. Default Cargo
and ordinary per-unit dynamic-plan outputs stay unchanged.

### Decision: Readiness is a typed Rust dependency requirement

**Choice:** Add exact producer-attempt readiness requirements to the existing
Rust pure model, separate from final confirmation. Logical phases are
interface-ready, full-metadata-ready, and link-artifact-ready. They are not
claims that these new CLI flags or wire variants already exist. All dependent
producers must satisfy the required phase before an effect becomes eligible.

- Analysis may use admitted early metadata for supported rlib dependencies.
- Code generation requires matching full metadata, including MIR needed for
  generic/inline instantiation.
- Linking requires the compiled libraries; cross-crate LTO requires those
  libraries before code generation.
- Consuming a proc-macro/dylib requires its completed executable library.
- Executing a build script requires its confirmed executable and all declared
  generated-output inputs. Merely compiling that script may use supported rlib
  interface dependencies, but must not run speculative output.

**Rationale:** One undifferentiated dependency edge over-serializes analysis;
one permissive early edge is unsafe for execution/code generation. Consts,
opaque types, async witnesses, and generic bodies must obey the compiler's real
metadata contract, not a hand-written public-signature scanner.

### Decision: Event-driven session, not repeated compiler actions

**Choice:** Extend existing Rust execution ports to carry bounded live artifact,
wait, resume, process-exit, and failure observations. Reuse the provisional
session's exact attempt binding, reservation accounting, held diagnostics,
confirmation, and owned teardown. Decoder observations produce pure eligibility
plans; the shell performs process/filesystem effects. Use canonical ready-unit
ordering with identity tie-breaks; do not put monotonic times or arrival order
in action identity.

Use one compiler process per admitted unit, with the supported metadata and
code emission in that process. Do not create separate analysis and build
commands that repeat the compiler's work. Compare serial whole-unit and normal
full-metadata pipelining against early-interface mode on the same supported
workloads; ordinary Cargo already pipelines full metadata.

**Rationale:** Early interfaces and metadata swap are compiler facilities. The
build tool cannot emulate them by publishing an incomplete `.rmeta`. Existing
buffering wrappers are incompatible until a bounded live-control path is
implemented or an explicitly admitted direct path is selected. No wrapper
bypass may skip required compiler policy, input admission, or cache authority.

### Decision: Final output authority is unchanged

**Choice:** Early files and locks stay within one admitted local session and
exact producer attempt. Verify matching full metadata identity before dependent
code generation, and reject a producer crash or missing/mismatched final file.
Commit only through confirmed success and existing artifact/cache checks. Never
mark a generic goal Done, produce signed PathInfo, or satisfy a dynamic-plan
`UnitOutput` because an early file appeared.

**Rationale:** The reference assumes colocated files/locks and concurrent local
producers. Remote/substituted final artifacts retain today's completed-output
path. Cross-sandbox or remote live readiness needs a separate transport and
authority design and is out of scope.

## Component ownership

Pure readiness, validation, ordering, and rejection decisions belong to
`mantle-rust-plan-core`; application orchestration belongs to
`mantle-rust-plan-app`. Existing CLI/provider/process/cache adapters own effects.
The offline Cargo helper owns only declared trial selection and bounded trial
evidence. Generic `crunch-build` scheduling/store goals acquire no Rust phase
meaning. Future implementation records these constraints in an ADR before
introducing an enabled mode.

## Alternatives considered

| Alternative | Disposition |
| --- | --- |
| Enable unstable flags globally on current nightly | Rejected: compiler capability, identity bug, and bounds remain unproven |
| Copy the compiler fork into the default lane | Rejected: unresolved correctness and maintenance cost |
| Put InterfacesReady into generic GoalState | Rejected: conflates speculative analysis with successful output authority |
| Stage one complete Rust session inside one sandbox | Selected first experiment; preserves existing publication boundary |
| Extend existing native Rust core/application after evidence | Selected second stage; no parallel planner convention |
| Share live files across per-unit sandboxes or remote workers now | Deferred outside this change; lacks a transport/admission contract |

## Verification, risks, and non-claims

Use the two prerequisite observation/differential packages and bounded-session
failure cases. Require real overlap evidence, default-hash incremental identity,
normal full-metadata swap at supported optimization levels, codegen/link/LTO
waits, proc-macro/script execution barriers, producer failure/crash, stale
artifacts, compatible wrapper behavior, and independent-root success. Explicitly
list supported target/profile/feature cells; missing required capabilities and
unrun cells are blocked, not green skips.

Record successful-run ranges and all failed/timed-out runs. A speedup claim
requires comparable retained cohorts and an observed improvement beyond their
reported variation; no fixed universal percentage is promised. Saturated/wide
builds may gain nothing and spend more CPU. Resident memory may increase.
Holding reservations is deliberately conservative. No result establishes
compiler correctness, full Cargo compatibility, global reproducibility,
bootstrap correctness, self-hosting, or release eligibility. No default-mode
cutover is authorized by this proposal.
