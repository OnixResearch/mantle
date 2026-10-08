# ADR 0091: Version long-lived service readiness states and restart policy

## Status

Proposed (2026-09-30). The vocabulary and schema are specified; producer integration, actual supervised restart observations, doctor publication, source-built proof-stage evidence, gates, and acceptance remain open.

## Context

Mantle's Rust cache daemon has a Unix socket and framed request protocol, remote serve exposes a metadata-only mode or one framed stdio session, and the source-built fixed-point plan orders six proof stages. These are different observations, not a shared readiness report. ADR 0080 assigns resident coordination to a separate daemon, but that daemon has not yet been implemented on the `origin/main` baseline. `mantle doctor` currently prints a prerequisite report, not service presence. Process creation, bound socket, and successful request must not be conflated; neither live state nor doctor checks constitute build or release evidence.

## Decision drivers

- A dependent component needs one declared, inspectable blocker rather than relying on incidental code order or a process-running flag.
- A restart must invalidate prior readiness, including facts published before a daemon failure.
- The restart matrix must be closed and each supervised component must name one policy, while proof stages are not falsely labeled supervised processes.
- Existing human and JSON `doctor` report shapes and evidence authority remain unchanged.

## Decision

Adopt the versioned `mantle-service-readiness-v1` declaration and report vocabulary in `.cairn/changes/unify-long-lived-service-readiness/specs/service-readiness/schema.json`. `started` records process/binding startup, and `ready` is an additional assertion only after the component handles a real request; `complete` and `failed` are terminal for the generation. A fresh restart generation cannot inherit `ready`. Declared user states cannot replace the built-in states. Admit a bounded, acyclic graph of explicitly named dependencies; derive readiness only when the component has asserted both `started` and `ready` and every declared dependency in the current snapshot is `ready` or `complete`. Name all blockers, including absent and failed dependencies. A proof stage becomes complete only after its real output and receipt validate; stage order by itself is not proof completion.

Start new dependents only after all declared dependencies are `ready` or
`complete`. An already-running dependent that loses readiness from a
retracted predecessor may remain started, but must retract its own `ready`.
The report carries effective states, so it must not expose raw local `ready`
assertions while the graph says blocked.

Require exactly one of `always`, `on-error`, `all`, `never` for each supervised
service; proof stages explicitly use a null policy. For a normal exit,
`always` restarts the component and the other three policies do not. For an
abnormal exit, `always` and `on-error` restart the component, `all` restarts
its supervised group, and `never` performs no restart. An exit before
readiness is `failed` (not `ready`) regardless of policy. For an abnormal
`never` exit, retain terminal `failed` rather than interpreting failure as
`complete`: unlike Synit's default convention, Mantle must preserve the
directly observed failure. The coordination daemon declares `never`: its
process has no supervising restart loop, and manual relaunch is a fresh
daemon epoch that must report `started` before any new `ready` assertion
after a real subscription response. The Rust cache daemon currently also
declares `never` without a supervisor; its proposed `on-error` assignment
requires a real supervised-exit fixture before changing that policy.
The one-shot remote stdio binding declares `never`; metadata-only remote
serve cannot claim a ready binding.

The report is explicitly `coordination-state` with `evidence_eligible: false`. Publish doctor-derived readiness through a separate optional channel when coordination exists, without adding fields to, or changing text from, the existing `crunch-doctor-report-v1` output. Evidence validators and receipt builders must reject this report as evidence, not merely ignore an unfamiliar property.

## Rejected alternatives

- Per-component implicit readiness or start-implies-ready: a blocked dependency and an early exit would be misreported as usable.
- Inferring dependencies from code order: it hides blocked and retracted predecessors.
- A general process supervisor: exceeds Mantle's build-tool boundary (ADR 0010).
- Reusing doctor JSON or release receipts to carry live readiness: changes established report semantics and conflates revocable coordination with durable evidence.
- Treating an abnormal `never` exit as `complete`: would hide a failure before the first real request.

## Consequences and non-claims

A pure decision core can validate and derive a deterministic bounded snapshot while shells retain authority over process, request, binding, policy-load, and proof-receipt observations. Producers and an actual coordination daemon must separately implement publication/retraction and exercise real requests and controlled exits. Proof stages may truthfully report blocked when their predecessors have no accepted output; a `complete` stage requires a validated output/receipt, not a successful preflight or an assumed fixed point. This ADR alone supplies no daemon, no proof run, and no passing readiness test. A `ready` snapshot does not prove continued availability, correctness of a build, a source-built fixed point, reproducibility, or release eligibility.
