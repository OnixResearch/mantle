# Design: One readiness vocabulary for long-lived Mantle services

## Goal and scope

Give four bounded consumers (coordination daemon, Rust cache daemon, remote
serve binding, source-built fixed-point proof gates) a common current-state
model. `mantle doctor` is an optional derived observer, not a consumer with
readiness authority. No general supervisor is introduced.

## Current behavior

The cache daemon admits requests after policy load and peer checking; remote
serve needs a usable binding; proof gates run after plan prerequisites.
Neither a process-start flag nor code order alone proves work can be taken.
The coordination daemon already has a local volatile fact surface (ADR 0088).

## Approach review

| Family | Mechanism | Disposition | Required behavior |
| --- | --- | --- | --- |
| Keep per-component readiness | Document each handshake | Rejected: no shared current set or explicit blockers | Dependency loss and recovery |
| Adopt reviewed Synit vocabulary | Two concurrent `started`/`ready` assertions plus `complete`/`failed`/declared states | Selected, within bounded Mantle scope | Successful real request before `ready` |
| General supervisor in Mantle | Supervise arbitrary user programs | Rejected: out of scope (ADR 0010) | No arbitrary program admission |
| One mutable enum for readiness | Replace `started` with `ready` at one fact ID | Rejected: loses Synit's simultaneous assertions | Two current distinct IDs |

## Contract and component ownership

- Pure core: strict versioned state validation, declared dependencies,
  blocked-by evaluation, restart-policy decision, and derived reports over
  current in-memory assertions.
- Shell: component lifecycle, successful real-request observation, binding and
  policy setup, and optional doctor publication. The building and evidence
  planes never consult a live fact for admission or proof.
- Surface: a `crunch_live_state_core::Fact` with kind `service_readiness` has
  identity `(owner, kind, subject)` and a bounded JSON `state` encoding
  **exactly one** assertion under schema `mantle-service-readiness-v1`.
  Its required fields are `schema`, `service_id`, `state`, `custom_states`,
  `dependencies`, `restart_policy`, `blocked_by`, and
  `coordination_state: true`. The core bounds an assertion to 1,536 bytes,
  service IDs to 64 ASCII identifier characters, eight declared custom states
  of at most 48 characters each, and eight dependencies. Separate stable
  subjects `service/<service_id>/started` and `service/<service_id>/ready`
  create distinct IDs. The JSON carries exactly one asserted state, not a
  latest-state slot or a JSON union of `started` and `ready`.

## Decisions

### Decision: `ready` is currently asserted in addition to `started`

**Choice:** A live service asserts `started` when startup begins and then
asserts `ready` *as another current fact* only after it can handle an actual
request. Retain both IDs while ready; retract `ready` on capability loss and
both on owner/session loss. A daemon restart clears the prior generation; an
observer discards both old assertions on disconnect/reset and waits for
republication, never interpreting an old `ready` as current.

**Rationale:** Synit explicitly models the union of *asserted* states, not a
single last-write-wins state. See
`/home/brittonr/.local/share/mantle-references/synit-book/pages/12-operation__service.md:57-76`
and [the guide](../../../docs/service-readiness.md).

### Decision: Dependencies are declared and reevaluated

**Choice:** Each assertion declares dependencies. The evaluator requires every
dependency currently `ready` or `complete` before allowing `ready`, publishes
`blocked_by` when one is not, and withdraws `ready` again on loss. Proof-stage
dependency declarations derive from existing signed-plan output references;
they do not rewrite signed plan bytes or digests. A protected StageX proof and
C cache policy retain their independent admission gates.

**Rationale:** Code order does not explain which dependency blocked a
component, or invalidate stale readiness when an owner goes away.

### Decision: Restart policy is the exact closed matrix

**Choice:** Require one of `always`, `on-error`, `all`, `never` in every
reported assertion. For normal/abnormal termination respectively: `always`
restarts this process / this process; `on-error` leaves it alone / restarts
this process; `all` leaves it alone / restarts the whole named daemon group;
`never` leaves it alone / leaves it alone. A `never` process that exits
abnormally **after readiness** reports terminal `complete` as Synit specifies.
Any process exiting **before readiness** reports `failed`, including under
`never`; this Mantle early-exit rule takes precedence only before readiness.
This matrix does not confer supervision authority over arbitrary programs.

**Rationale:** The source matrix is
`/home/brittonr/.local/share/mantle-references/synit-book/pages/18-operation__builtin__daemon.md:68-89`;
Synit's `all` means a whole daemon group, not all Mantle services.

### Decision: Admission and doctor remain bounded and non-authoritative

**Choice:** Daemon ingress checks state schema and bounds, known/declared
state, exact required policy, one active owner per process session, distinct
stable subjects, and no `ready` before `started` or while dependencies block.
Unknown schema/state, unknown/missing policy, or contradictory assertions are
rejected rather than rendered as readiness. Session close retracts both IDs.
Doctor may publish a transient **derived** `coordination_state` report only
when a daemon is available; preserve exact existing human/JSON output and
evidence/receipt bytes. No additive doctor stdout fields.
One-shot best-effort coordination transport is capped at 100 ms, and an absent
or nonreading endpoint cannot block builds or alter receipts. Remote
stdio/stdout/stderr frames never carry readiness; use the separate
coordination channel.

## Risks / Trade-offs

- A new state's label can drift from real behavior. Require a successful real
  request before `ready`; a lifecycle flag or binding existence alone cannot
  promote it.
- A lost owner, disconnected subscriber, or daemon restart can leave a stale
  observer view if reset and retraction are ignored. Treat only current
  generation facts as assertions; re-evaluate blockers after any loss.
- A dependency cycle or contradictory declaration cannot yield readiness;
  reject at ingress rather than waiting silently.
- Doctor publication is optional and transient: keep human/JSON output and
  evidence/receipt bytes exactly unchanged, even when the daemon is absent.

## Non-Claims

- A current `ready` assertion describes observed capability to take work; it
  does not prove future availability.
- No live readiness fact is proof of output quality, correctness, protected
  StageX eligibility, C cache policy admission, or release eligibility.
