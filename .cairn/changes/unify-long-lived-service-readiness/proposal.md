# Proposal: One readiness vocabulary for long-lived Mantle services

## Why

Mantle describes readiness three ways. Goal transitions use their own state
names. The Rust cache daemon announces startup by protocol handshake. Proof
gates encode prerequisites as code order. A dependent component therefore has
no shared way to wait for "this component is ready", and an operator has no
single report that answers which long-lived components are up.

The Synit manual declares service state with a small vocabulary (`started`,
`ready`, `complete`, `failed`, plus user-defined values), declares dependencies
with `depends-on`, and names restart policies (`always`, `on-error`, `all`,
`never`) (`~/.local/share/mantle-references/synit-book/pages/12-operation__service.md`,
reviewed in `docs/synit-application-notes.md`). Its `ready` state is asserted in
addition to `started`, so a consumer can distinguish "process running" from
"able to take work".

Mantle's long-lived components already have real readiness concepts: the cache
daemon accepts requests only after admission policy loads, a remote serve
binding is usable only after the binding exists, and a proof stage is usable
only after its predecessor is complete.

## What Changes

- Define one readiness vocabulary: `started`, `ready`, `complete`, `failed`,
  and declared user-defined states. A component MUST be able to report
  `started` and `ready` separately.
  r[mantle.service_readiness.readiness_vocabulary]
- Declare dependencies between long-lived components and declared proof gates.
  A component MUST NOT report `ready` before its declared dependencies report
  `ready` or `complete`.
  r[mantle.service_readiness.declared_dependencies]
- Name the restart policy for each supervised component from the reviewed
  matrix: `always`, `on-error`, `all`, `never`. The policy MUST appear in the
  component's runtime report.
  r[mantle.service_readiness.restart_policy_matrix]
- Make `mantle doctor` publish derived readiness state in addition to its human
  and JSON output. Derived state MUST be marked as coordination state and MUST
  NOT be accepted as evidence.
  r[mantle.service_readiness.doctor_derived_state]
- Apply the vocabulary first to four bounded consumers: the coordination
daemon, the Rust cache daemon, the remote serve binding, and the source-built
fixed-point proof stages. The coordination daemon reports its own readiness
under the same vocabulary (ADR 0080).

## Impact

- **Immediate consumer**: the coordination daemon, operator diagnostics, the
  Rust cache daemon, remote serve, and proof-stage gating.
- **Immediate outcome**: one question has one answer: which declared
  components are ready, failed, or complete.
- **Durable capability**: a readiness model that later carries worker presence
  and demand facts on the coordination surface.
- **Maintenance owner**: Mantle operator diagnostics owner, with the daemon and
  remote owners for their components.
- **Repeatability evidence**: readiness fixtures per component, dependency
  fixtures, restart-policy matrix fixtures, and doctor state rendering.
- **Compatibility**: `doctor` keeps its current human and JSON output. New
  state is additive and versioned.

## Scope

The change covers the vocabulary, dependency declarations, restart-policy
assignment, doctor state, and the three named consumers.

## Out of Scope

- A general-purpose system service supervisor.
- Process supervision outside the named components.
- Service discovery, socket activation, or service dependencies for arbitrary
  user programs.
- Treating readiness state as build or release evidence.

## Success Criteria

- A dependent component waits for `ready`, not for process start.
- A process that exits before announcing readiness reports `failed`.
- Each of the three consumers reports the declared restart policy.
- Doctor state renders without changing existing doctor output shapes.
