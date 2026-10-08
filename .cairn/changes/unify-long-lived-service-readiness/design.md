# Design: One readiness vocabulary for long-lived Mantle services

## Goal and scope

Give the three named service surfaces (coordination daemon, Rust cache daemon,
remote serve binding) and the six declared proof stages one readiness model
with explicit dependencies and service restart policy. No general supervisor.

## Current behavior

The Rust cache daemon serves requests after a policy load and a peer check. A
remote serve binding accepts a session after binding setup. Proof stages run in
declared order. Each site expresses readiness its own way, and no shared report
answers which components are ready.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Keep per-component readiness | Document each handshake | Rejected: no shared vocabulary, dependencies stay implicit | Dependency fixture |
| Adopt the reviewed vocabulary | `started`/`ready`/`complete`/`failed` plus declared states | Selected direction | Readiness and dependency fixtures |
| General supervisor in Mantle | Own process supervision for user programs | Rejected: out of scope for a build tool (ADR 0010) | Not blocking |
| Readiness in the coordination surface only | Publish facts, keep no local states | Rejected: the daemon needs local decisions too | Daemon fixture |

## Contract and component ownership

- Pure core: admit versioned declarations and observations, validate state and
  policy, evaluate the declared dependency graph, and derive a bounded report
  without filesystem, clock, process, or network effects.
- Shell: observe component startup, successful real request, exit, binding,
  and proof-stage receipt; publish a projection to the coordination surface.
- Reporting: the projection is coordination state, never PathInfo, an
  attestation, a build receipt, or release evidence. Doctor's existing
  `crunch-doctor-report-v1` JSON and human text remain byte-shape compatible;
  publication is a separate opt-in output channel, not another doctor field.
- The proposed versioned interchange contract is
  `specs/service-readiness/schema.json`, schema
  `mantle-service-readiness-v1`. This records the target wire shape, not an
  implemented producer or validator.

`src/bootstrap_validate.rs` persists the current doctor JSON in bootstrap
validation evidence, while `src/build_plan.rs` consumes the current preflight
report as a planning check. Neither site should silently acquire
coordination-state fields or treat a published readiness snapshot as proof;
publication must be separate from `PreflightReport::render_json()`.

## Versioned readiness contract (T1.2 decision)

The input has `schema`, `components`, and `observations`. A component declares
one unique nonempty `id`, `kind` (`service` or `proof-stage`), a list of distinct
`depends_on` IDs, `user_states`, and exactly one `restart_policy` for a
supervised service (`always`, `on-error`, `all`, `never`). A proof stage is not a
supervised process: its policy is explicitly `null`, not a missing policy.
Every proof stage also declares `stage_requirements` with explicit
`require_action_reconciliation` and `require_v2_receipt` flags; services use
`stage_requirements: null`. The pure core checks these flags against
shell-supplied completion facts, but it never reads or authenticates receipts.
Reject unknown fields, states, policies, duplicate IDs, self-dependencies,
unknown dependencies, dependency cycles, and observations naming undeclared
components or user states. Sort component IDs and blockers for reproducible
reports; cap graph size before work. Missing runtime observations are not
fabricated `started` states.

An observation binds a component ID and a generation to asserted states and
an explicit `exit` (`null`, `normal`, or `abnormal`). `started` records the
running process or created binding; `ready` is **additional** to `started`
and follows a successful real request (or a verified subscription response
for the coordination daemon), not merely socket existence or policy load.
`complete` and `failed` are terminal for that generation: retract `started`
and `ready` on exit, and never co-assert a terminal state with `ready`.
On exit before readiness, report `failed`, even after a normal exit or
under `never`; exit class still controls restart. Each restart creates a
new generation with no inherited assertions; a previous generation's
`ready` cannot satisfy a dependent. User-defined states must be declared
on the component and cannot substitute for `ready` or `complete`.

An observation carries `request_acknowledged` and `proof_completion` separately
from local state strings. For services, `ready` without an acknowledged
successful real request is rejected, even if the process has started or the
socket exists. For proof stages, `complete` requires shell-verified execution,
nonempty exact BLAKE3 output and stage-evidence digests, and all declared
action/receipt requirements; `ready` is not fabricated for one-shot stages.
The shell must validate actual bytes, signatures/receipts, and action-event
reconciliation before setting these facts. The existing v2 proof receipt
validator alone does not bind complete action plans/events; it cannot justify
`complete` until the fixed-point owner closes that authority.

The derived report has `schema`, `classification: coordination-state`,
`evidence_eligible: false`, and ordered components. Each row includes ID,
generation, effective states, declared restart policy, the matrix-derived
`restart_action` (`null` before exit, otherwise `none`, `component`, or
`group`), `ready` (a boolean derived from local observation AND dependencies),
and ordered `blocked_by` IDs. The evaluator MUST omit `ready` from both
effective states and the boolean while any dependency is blocked, even if
the component's own local request has succeeded. A *new* dependent process
MUST wait to start until each dependency is `ready` or `complete`. An
already-started process may remain `started` after its dependency retracts,
but must retract `ready`. A proof stage can advance only after its
predecessor has *validated* completion, not just a preceding command's exit.
An absent or failed dependency is named as a blocker. `failed` never becomes
derived-ready. Proof-stage rows always use `restart_policy: null` and
`restart_action: null`; completion or blockage never schedules a process
restart. This report is a snapshot, not durable proof of continued
availability; consumers must re-evaluate after retraction, exit, or restart.

| Policy | Normal exit after readiness | Abnormal exit after readiness | Normal exit before readiness | Abnormal exit before readiness |
| --- | --- | --- | --- | --- |
| `always` | `complete`, restart component | `failed`, restart component | `failed`, restart component | `failed`, restart component |
| `on-error` | terminal `complete` | `failed`, restart component | terminal `failed` | `failed`, restart component |
| `all` | terminal `complete` | `failed`, restart supervised group | terminal `failed` | `failed`, restart group |
| `never` | terminal `complete` | terminal `failed` | terminal `failed` | terminal `failed` |

The `never` abnormal-exit result deliberately preserves the observed
failure (the proposal's early-exit rule), rather than copying Synit's
abnormal-exit-as-`complete` convention. The matrix decides *whether and
what to restart*; it cannot turn an observed failure into evidence of
completion. Controlled exits and actual supervised restarts must verify
each cell before consumer acceptance.

When a policy restarts, publish a terminal `failed` or `complete` observation
for the exiting generation before retracting it and publishing the next
generation's `started`; a later current-state snapshot alone cannot prove
the early-exit transition. Restart admission is denied until old `ready` is
retracted. Consumers observing transitions may see the blocker explicitly.

The isolated pure core exposes `evaluate(snapshot)` for deterministic
projection and `advance(previous_report, snapshot)` for ordered transition
admission. A production shell must preserve the prior report and call the
transition path: an initial/new generation first reports `started` without a
request acknowledgment, a blocked dependent cannot start, a previous
generation must publish a terminal exit before replacement, a terminal state
cannot revive in place, and the policy must permit the restart. A single
snapshot cannot establish that event order or authenticate its shell facts.
Keep the terminal observation until its replacement generation is published
or the service declaration is explicitly removed; dropping an observation
while keeping its declaration is not an observed exit or a policy-authorized
restart.

The current `mantle remote live serve` has **no supervising restart loop**.
SIGINT/SIGTERM stop it; SIGKILL requires a separate manual relaunch. Its
declared policy is therefore `never`, not an unimplemented automatic
`always`. A manual relaunch begins a fresh daemon epoch with no inherited
facts; it must report `started` again and cannot report `ready` before an
actual subscription responds through both `snapshot-start` and
`snapshot-end`. Process death retracts prior readiness; a new socket or
stderr `listening` is insufficient. This does not claim automatic recovery.

Current declared assignments: coordination daemon `never`, Rust cache daemon
`never`, and remote `stdio-once` binding `never`. Neither daemon has a
supervising restart loop; a future cache `on-error` assignment remains proposed
until a real supervisor and controlled exit/restart fixture are accepted.
Metadata-only remote serve is not a binding and cannot assert readiness.
For `stdio-once`, the binding can assert readiness only after an
accepted, successful framed session, then immediately retract it on normal
completion. The metadata command never starts a service. Do not turn its
advertised transport list into a live listener or prolong the one-shot
process to manufacture a long-lived state.

The six fixed-point proof stages form a declared DAG, not services:
`stagex-transition` -> `stagex-provider-publication` ->
`full-source-native-provider` -> `full-source-rust-provider` ->
`mantle-stage1` -> `mantle-stage2`. Stage1 additionally needs native and
Rust provider outputs; Stage2 additionally needs both providers and Stage1.
Source/policy authority inputs remain under the proof-plan contract, not
fabricated readiness nodes. No stage reports `complete` without its real,
validated output/receipt.

For this six-stage graph each stage requires stage-local protected/build action
reconciliation against the preexecution root plan before `complete`. Stage1
additionally requires its persisted output identity, successful execution and
smoke; that is only a stage-local predecessor fact, not a fixed-point claim.
Stage2 requires the global v2 receipt after stage2 output identity, binary
digest equality, and all six-stage event reconciliation. Requiring the global
v2 receipt before Stage1 can start Stage2 would be a dependency cycle.
The present proof shell lacks accepted stage-local action observations, so
it cannot assert `complete`. Its verbose coordination projection begins with
absent/blocked rows and may show StageX `started` only inside actual
isolated execution; a successful StageX command retracts to absent until
the missing action authority is available.

For the measured baseline invocation missing `--source-profile`,
`stagex-transition` has no authenticated source authority and cannot become
`complete`. The CLI reports an input-admission failure, not a fictitious
upstream service component. `stagex-provider-publication` can then name
`stagex-transition` as its unavailable declared component dependency;
later stages name their corresponding unsatisfied predecessors. The
coordination projection can truthfully emit blockers without making a
fixed-point claim; it must not turn source-plan existence, command launch,
or an unverified output into `complete`.

## Fixture acceptance matrix (not yet executed)

| Case | Observation required | Expected contract result |
| --- | --- | --- |
| Positive request | Cache/coordination/remote binding starts; a real framed request or subscription succeeds | Only then assert both `started` and `ready` for the same generation |
| Positive dependency | `b` depends on `a`; `a` only `started`, then handles a real request | `b` names `a` as blocker until `a` is `ready` |
| Positive exits | Controlled normal and abnormal exits under each of the four policies | Exact restart scope and terminal/failed state from the matrix |
| Negative early exit | Process exits before first successful request | `failed`, no `ready`; restart policy still determines next action |
| Negative blocked proof | StageX transition lacks authenticated source/receipt | Provider and subsequent stages name predecessor blockers; no fixed-point completion |
| Negative admission | Undeclared state, missing service policy, unknown dependency, duplicate ID, cycle | Admission error; never a partial derived report |
| Negative evidence | Doctor publishes separate `coordination-state` report | Existing JSON/human doctor outputs unchanged; evidence validators reject state and receipts remain unchanged |

These are acceptance fixtures to run against live adapters and the pure core,
not successful tests or a substitute for actual source-built stage receipts.

## Decisions

### Decision: `ready` is asserted in addition to `started`

**Choice:** A component reports `started` when its process or binding exists
and `ready` only when it can take work.

**Rationale:** The distinction is the whole point of a readiness model. The
reviewed manual keeps `ready` as an additional assertion for the same reason.

### Decision: Dependencies are declared, not inferred

**Choice:** Each component declares what it depends on. The evaluator does not
infer dependencies from call order.

**Rationale:** Inferred order is what the code already does. Explicit
declarations make a blocked component visible instead of silent.

### Decision: Restart policy is a closed matrix

**Choice:** Every supervised component names one of `always`, `on-error`,
`all`, `never`.

**Rationale:** A closed set is reviewable. The reviewed matrix already covers
the cases Mantle needs: restart on failure, restart always, treat all exits as
normal, or restart the whole daemon.

## Risks / Trade-offs

- New states can drift from real behavior. The fixtures check readiness against
  an actual request, not a flag.
- Opt-in coordination output can drift into doctor reports; keep its versioned
  side channel separate from the existing human and JSON output shapes.
- A blocked dependency must not stall silently; the report names the blocking
  component.

## Non-Claims

- Declared readiness proves the component's own observation.
- A `ready` component is not proof of output quality, correctness, or release
  eligibility.
