# ADR 0086: Use declared, current service-readiness states

## Status

Accepted (2026-10-06). This is a bounded contract decision, not a claim that consumer acceptance or an end-to-end build comparison has run.

## Context

Mantle's coordination daemon, Rust cache daemon, remote serve binding, and source-built fixed-point proof gates need a shared answer to whether a component has started, can actually take work, is blocked, or is terminal. Process existence and execution order cannot provide that answer. `mantle doctor` may summarize current assertions but cannot confer admission or evidence authority.

The reviewed Synit service text at `/home/brittonr/.local/share/mantle-references/synit-book/pages/12-operation__service.md:49-76` defines `depends-on`, says service state is the *union of currently asserted states*, and specifically says `ready` is asserted **in addition** to `started` (lines 64-70). The reviewed restart text at `/home/brittonr/.local/share/mantle-references/synit-book/pages/18-operation__builtin__daemon.md:68-89` defines exactly `always`, `on-error`, `all`, and `never`. The operator interpretation is in [the readiness guide](../docs/service-readiness.md). These are source references, not a claim of Synit runtime integration.

## Decision drivers

- Distinguish existence from ability to handle a real request without turning a live observation into proof.
- Make dependencies and blocking identities visible, including subsequent dependency loss.
- Preserve volatile publisher/session ownership and generation reset of ADR 0088.
- Keep builds, proof admission, cache policy, remote frames, and doctor output independent of optional coordination availability.
- Restrict restart decisions to the four named consumers, not arbitrary user programs.

## Decision

Use a pure readiness contract and the existing bounded `service_readiness` fact kind. A ready service publishes **two simultaneously current facts**: `started` at `service/<service_id>/started` and `ready` at `service/<service_id>/ready`. `crunch_live_state_core::Fact::new(owner, FactKind::ServiceReadiness, subject, state)` gives them distinct IDs because fact identity is `(owner, kind, subject)`. Neither a single overwritten enum nor a JSON union of both states satisfies this contract. Publish `started` when startup begins; publish `ready` only after the component has handled a real request and all declared dependencies are currently `ready` or `complete`. A successful bind or protocol startup handshake alone cannot imply readiness. Retract `ready` on capability/dependency loss; name a blocker with `blocked_by`. Early process exit before readiness reports `failed` (including under `never`) and never spuriously reports `ready`.

The `Fact.state` value is a bounded JSON encoding of **one** assertion under `schema: "mantle-service-readiness-v1"`, not a plain state string. Required fields are `schema`, `service_id`, `state`, `custom_states`, `dependencies`, `restart_policy`, `blocked_by` (null or a dependency ID), and `coordination_state: true`. Built-in states are `started`, `ready`, `complete`, and `failed`; other states must be explicitly declared. The assertion is limited to 1,536 bytes; a service ID is at most 64 ASCII identifier characters, with at most eight dependencies and eight declared custom states (at most 48 characters each). Daemon ingress validates the schema, one-state/subject identity, declared state and dependencies, exact required policy, and current `started`/dependency prerequisites before admitting `ready`; it rejects unknown schemas/states, unknown or missing policy, contradictory declarations, and invalid/oversize data. One active publisher session has a unique owner; a second live session cannot take that owner. Session close retracts that owner's current assertions, including both `started` and `ready`. A daemon restart loses all facts; subscribers discard the prior generation on EOF/reset and require fresh assertions, never carrying old readiness forward. See [the live-state guide](../docs/live-build-state.md) for publisher/observer generation behavior and daemon-wide caps.

Each of the four consumers declares dependencies. Source-built fixed-point proof-stage dependencies derive from the *existing signed-plan output references*: this is observation and gating over the existing plan, not a rewrite of plan bytes or digests. No readiness assertion activates protected StageX proof or C cache policy. Remote serve uses the separate coordination channel; stdio frames, stdout, and stderr never transport readiness.

The closed restart policy applies only to the bounded managed component lifecycle:

| Policy | Normal termination | Abnormal termination |
| --- | --- | --- |
| `always` | Restart this process, leaving peers alone | Restart this process, leaving peers alone |
| `on-error` | Do not restart | Restart this process, leaving peers alone |
| `all` | Do not restart | Restart the whole named daemon group |
| `never` | Do not restart; terminal `complete` | Do not restart; Synit treats this as terminal `complete` |

Mantle gives the early-exit-before-ready `failed` rule precedence over Synit's `never` treatment of abnormal termination as `complete`. `all` never means all Mantle processes. Every assertion declares one of the four policies; no implicit default or general-purpose service supervisor is introduced.

Doctor may publish a **transient derived** `coordination_state` report only when a daemon is available. Exact existing human/JSON output bytes and evidence/receipt bytes stay unchanged; no doctor stdout field is added. An unavailable or nonreading endpoint degrades observation only: one-shot best-effort publication has a 100 ms cap, cannot block builds, and cannot change their outputs or receipts. A live readiness fact or doctor report is never build/store/release evidence or an admission input.

## Rejected alternatives

- **One mutable service-state fact or last enum value:** overwriting `started` with `ready` destroys the concurrent assertion from the reviewed Synit semantics.
- **Unbounded supervisor or arbitrary user-service discovery:** unnecessarily changes Mantle's process and trust boundaries.
- **Doctor output extensions or transport through remote stdio:** breaks existing output/remote protocol contracts and confuses diagnostics with evidence.
- **Proof authority from a current fact:** ephemeral owner-scoped assertions cannot replace signed plan references, independent protected-proof gates, or canonical receipts.

## Consequences and non-claims

Consumers can observe a blocked → ready → blocked transition and session loss without treating startup as readiness. This costs two fact IDs while ready and requires strict ingress checks and retraction. Readiness describes current observed capability only; a request can fail later. The decision does not prove that all four consumer integrations, restart behavior, doctor byte parity, or a CLI build with/without coordination have passed tests. No release eligibility or evidence authority follows from it.
