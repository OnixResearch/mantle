# Proposal: One readiness vocabulary for long-lived Mantle services

## Why

Mantle describes readiness across four bounded consumers. The coordination
daemon maintains live facts, the Rust cache daemon admits work after its policy
loads, remote serve accepts work after its binding is usable, and source-built
fixed-point proof gates have signed-plan predecessors. Process existence alone
does not answer whether a dependent can take work. `mantle doctor` is an
optional derived observer, not a fifth readiness authority.

Synit declares service state and dependencies in
`/home/brittonr/.local/share/mantle-references/synit-book/pages/12-operation__service.md:49-76`
and its daemon restart matrix in
`/home/brittonr/.local/share/mantle-references/synit-book/pages/18-operation__builtin__daemon.md:68-89`
(reviewed in [the operator guide](../../../docs/service-readiness.md)).
Synit's `started` and `ready` are *two concurrent assertions*, not successive
values of one state: `ready` is asserted in addition to `started`. Mantle keeps
that distinction in distinct, stable service-state fact identities.

## What Changes

- Define one readiness vocabulary: `started`, `ready`, `complete`, `failed`,
  and declared user-defined states. A ready component MUST concurrently assert
  `started` and `ready` as separate current facts, with distinct stable
  subjects/IDs. `ready` follows successful handling of a real request.
  r[mantle.service_readiness.readiness_vocabulary]
- Declare dependencies between the four consumers and proof gates. A
  component MUST NOT report `ready` before declared dependencies report
  `ready` or `complete`; it names blocking dependencies and withdraws `ready`
  on dependency loss. Proof-stage dependencies come from signed-plan output
  references, without changing plan digests.
  r[mantle.service_readiness.declared_dependencies]
- Name the restart policy for each supervised component from the exact closed
  matrix `always`, `on-error`, `all`, `never`. Each runtime assertion carries
  bounded versioned state with its policy, dependencies, and a
  coordination-state marker; unknown or missing policy/state is rejected.
  r[mantle.service_readiness.restart_policy_matrix]
- Let `mantle doctor` publish a transient derived coordination-state report
  only when the daemon is available. It MUST preserve exact existing human
  and JSON output bytes and evidence/receipt bytes, MUST NOT add doctor
  stdout fields, and MUST NOT turn coordination state into evidence.
  r[mantle.service_readiness.doctor_derived_state]
- Apply the vocabulary to exactly four bounded consumers: the coordination
  daemon, Rust cache daemon, remote serve binding, and source-built
  fixed-point proof gates. The coordination daemon reports its own readiness
  under the same vocabulary (ADR 0080).

## Impact

- **Immediate consumers**: the coordination daemon, Rust cache daemon, remote
  serve, and source-built fixed-point proof gates; doctor is an optional
  diagnostic side channel.
- **Immediate outcome**: current, retractable declarations of which
  components are started, ready, failed, complete, or blocked.
- **Durable capability**: a readiness model that later carries worker presence
  and demand facts on the coordination surface.
- **Maintenance owner**: Mantle operator diagnostics owner, with the daemon and
  remote owners for their components.
- **Review criteria**: real-request readiness, dependency loss and recovery,
  early exit, invalid schema/policy, exact restart matrix, daemon restart, and
  doctor/evidence separation; these are required behaviors, not claims that
  acceptance has run.
- **Compatibility**: doctor human/JSON output bytes and evidence/receipt
  bytes remain exactly unchanged. Missing or nonreading coordination endpoints
  never block builds or change receipts; one-shot best-effort publication has
  a 100 ms cap.

## Scope

This change covers vocabulary, dependency declarations, restart-policy
assignment, and the four bounded consumers, with doctor as an optional
derived diagnostic channel.

## Out of Scope

- A general-purpose system service supervisor.
- Process supervision outside the named components.
- Service discovery, socket activation, or service dependencies for arbitrary
  user programs.
- Activating StageX/protected proof or C cache policy through readiness.
- Treating coordination readiness as build or release evidence.

## Success Criteria

- A dependent waits for `ready`, not only process start, and blocks again if
  its dependency's current readiness is retracted.
- A process exiting before readiness reports `failed`, regardless of `never`
  treating abnormal termination as complete in the reviewed Synit matrix.
- All four consumers declare policy; the exact four-case restart behavior is
  enforced only within Mantle's bounded component lifecycle.
- Doctor leaves existing human/JSON output and evidence/receipt bytes unchanged.
