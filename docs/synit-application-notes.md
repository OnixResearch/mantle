# Synit application notes for Mantle

Status: review notes, not a decision. This document records what the Synit
manual says, where Mantle already approximates the same design, and which
changes are worth making. Nothing here is implemented.

Source: *The Synit Manual* (<https://synit.org/book/>), licensed CC BY 4.0,
copyright © 2021–2023 Tony Garnock-Jones. Local saved copy:
`~/.local/share/mantle-references/synit-book/` (`print.html`, 53 per-page
markdown files, `synit-book.md`, `README.md` with BLAKE3 digests).

## 1. What the book documents

Synit is an operating system layer built on the Syndicated Actor Model (SAM).
The manual describes:

- **assertions with lifetime.** An actor publishes part of its state as an
  assertion. The assertion retracts automatically when its owner stops, crashes,
  or withdraws it. Failure is signalled by retraction, not by an error reply.
- **dataspaces.** An assertion goes to a dataspace. Subscribers publish
  interest as `Observe(pattern, ref)` assertions. The dataspace routes matching
  assertions and forwards retractions. Equal assertions deduplicate.
- **capability attenuation.** A capability is a reference plus a chain of
  caveats. Each caveat is a pattern rewrite, a pattern reject, or an alternative
  list. Caveats run right to left. A rejected assertion or message is dropped
  silently.
- **gatekeeper and sturdyrefs.** A gatekeeper entity upgrades a long-lived
  certificate to a live reference. The certificate is an iterated keyed HMAC
  over the object id and its caveats (macaroon style). Revocation is a
  retraction.
- **declared service state.** `require-service`, `run-service`, `depends-on`,
  `service-state` (`started` / `ready` / `complete` / `failed`), restart
  policies, milestones, and a configuration watcher that reloads changed
  scripts and drops their previous state.
- **Preserves and schemas** as the data language and type layer.
- **tracing** as one record per actor activation, with a causal `cause` field
  and an action taxonomy (`spawn`, `link`, `facet-start`, `enqueue`, ...).

The manual also states one deliberate omission: the SAM provides no
protocol-level failure feedback, because a live peer cannot distinguish a slow
peer from a dead one. Components must agree on a conversational frame first,
and debugging aids stay outside the model.

## 2. The load-bearing ideas for a build system

Five mechanisms carry the design. Each has a Mantle counterpart or gap.

1. **Fate-sharing.** State lives exactly as long as its owner. A worker that
   dies takes its claims with it. Mantle uses timeouts, expiry, and generation
   fences instead. Those work, but they turn liveness into a clock question.
2. **Interest-driven routing.** Demand is an assertion. Data flows to declared
   interest. Mantle selects work from a computed graph, and consumers poll
   reports.
3. **Attenuation as the grant.** A restricted reference *is* the permission.
   No policy re-check is required of the holder, and no feedback leaks to a
   rejected sender. Mantle issues fenced leases and bearer tickets and
   re-checks admission centrally.
4. **Name upgrade.** A durable name (`sturdyref`) becomes a live handle
   through one resolve step, and revocation is a retraction. Mantle has
   bindings, substitutes, and base layers, each with its own admission path.
5. **Declared state and readiness.** `started`, `ready`, `complete`, and
   `failed` are separate facts. Dependencies are declared, not inferred.
   Mantle has goal states, daemon readiness, and proof gates, but no shared
   vocabulary across them.

## 3. Mantle today

Mantle already implements a coordination system. The following table names the
current machinery.

| Area | Current machinery |
|---|---|
| Scheduling | `crunch-build::goal`: `AwaitingDerivation`, `Pending`, `Waiting`, `Ready`, `Building`, `Done`, `Failed`; waiter lists; dedupe by store identity; `MAX_GOALS` bound; deterministic lazy ready ordering (ADR 0017). |
| Remote work | Fenced leases, durable attempt state, receiver-probed locality, transfer checkpoints, one-use tickets (ADR 0025, `docs/remote-transfer.md`). |
| Store authority | Capability-split Rust values (`docs/store-authority-capabilities.md`), signed PathInfo, trust keys, concrete capability views (ADR 0058). |
| Store composition | Read-only base layers with descriptor and generation revalidation (ADR 0012). |
| Retention | GC plans with explicit retained roots, `legacy-unmanaged` migration, `state_dir/gc-roots.json` (ADR 0064). |
| Observation | `mantle-evaluation-stream-v1` NDJSON, `--json build` aggregate report, transcripts, remote telemetry, remote trace context. |
| Authority | Seccomp `execve` supervisor with absolute-path and BLAKE3 binding, protected stage proofs, release attestations and witness quorum. |
| Daemons | Rust cache daemon with `SO_PEERCRED` admission, policy file, receipt directory, drain-on-signal shutdown. |

Two gaps repeat through every row: **state has no lifetime** (expiry, TTL,
generation), and **observation has no subscription** (snapshots and one-shot
streams).

## 4. The organizing rule: three planes

Adopt Synit semantics in the coordination plane only. ADR 0080 separates build
coordination from batch building, so Mantle has three planes.

- **Building plane** (batch, deterministic): evaluation, conversion,
  scheduling, sandbox execution, store admission, retention records, and
  receipts. It emits bounded facts. It never requires the coordination plane.
- **Coordination plane** (reactive, resident daemon): subscriptions, worker
  and service presence, declared demand delivery, readiness aggregation,
  remote entry-point resolution, and grant issuance. Facts here may appear,
  change, and retract. The daemon never mutates the store and never authors
  evidence.
- **Evidence plane** (canonical): signed PathInfo, action results,
  attestations, transcripts, receipts, release bundles. Immutable, replayable,
  never retracted.

Two rules keep the planes apart:

1. A live assertion must not be quoted as evidence. Its meaning depends on its
   lifetime, so it is not reproducible from stored facts.
2. An evidence artifact must not carry a live handle. A receipt that names a
   session handle cannot be replayed later.

Three more rules keep the daemon optional and outside build authority:

1. A build runs with no daemon. Absence degrades observation only.
2. A receiver enforces a grant locally. Admission never waits for a daemon
   round trip.
3. Coordination state retracts on daemon restart. Nothing in the daemon is
   durable truth.

This rule also explains why Mantle's existing "receipts are not authority"
language is correct and should not be relaxed in the name of reactivity.

## 5. Chapter-by-chapter mapping

| Book topic | Mantle surface today | Verdict | Candidate |
|---|---|---|---|
| SAM assertions and retraction (`syndicated-actor-model.html`) | Goal state machine, waiter notification | Adapt: publish goal facts as assertions; keep the machine authoritative | C3 |
| Fate-sharing and automatic cleanup (`glossary`: facet, linked task) | Worker loss handled by lease release and generation fences | Adopt where a live session exists: session end releases claims immediately; keep expiry as the crash fallback | C6 |
| Dataspaces (`dataspace.html`, `dataspacePatterns.html`) | No equivalent; polling and reports | Reuse, do not rebuild: Molten (`aspen/`) owns dataspaces in this stack | C3 |
| Pattern matching with minimum-match group patterns | Exact decoding of plan and receipt types; nominal typing | Adapt narrowly: use minimum-match patterns only at declared extension points, never in identity | C9 |
| Capability attenuation (`protocol.html`: caveats) | Fenced leases, bearer tickets, capability views, trust keys | Adapt: pattern caveats for attenuated build and store authority; coordinate with UCAN and Basalt before new crypto | C7 |
| Gatekeeper and sturdyrefs (`builtin/gatekeeper.html`) | `cmd_remote_serve --binding`, substituters, base layers, credential files | Adapt if the farm grows: one resolve step per entry point, revocation by retraction | C8 |
| `require-service` / `depends-on` / `service-state` (`service.html`) | Goal states, daemon readiness, proof gates | Adopt the vocabulary and the dependency declarations | C5 |
| Restart policies (`builtin/daemon.html`) | Remote attempt policy, daemon drain | Adopt the four policies as a named matrix | C5 |
| Configuration watcher and reload semantics (`builtin/config-watcher.html`) | Nickel evaluation on each command; dev cache for the fixed-point proof only | Adopt: `mantle watch` re-evaluates and retracts removed goals | C2 |
| Attenuated config dataspace (rewrite `require-service`) | Project scoping in `mantle-project` | Adapt: hand each project a rewritten, namespaced view of build authority | C7 |
| Service objects and presence (`service-object`) | Worker handles, cache endpoints | Adopt presence assertions for worker and cache availability | C3, C5 |
| User settings as digest-named assertion files (`synit-config.html`) | `gc-roots.json`, overlay trust-key lists | Adopt for retention: one record per owner, digest-named, deterministic merge | C1 |
| Trace records with causes (`protocols/syndicate/trace.html`) | Transcripts, remote trace context, build logs | Adopt the causal vocabulary for a build trace | C4 |
| Milestones as pseudo-services | Proof gates expressed as code order | Adopt milestone facts for long proofs | C5 |
| `default-route`-style derived facts (`system-layer.html`) | `mantle doctor` one-shot checks | Adopt: doctor publishes derived state instead of only printing it | C5 |
| Silent discard and no protocol feedback (`syndicated-actor-model.html` note 6) | Fail-closed admission, "awaiting admission" states, non-claims | Already aligned: keep it, and cite the FLP-style rationale | — |
| Membranes and transient references (`protocol.html`) | Manifest precedes chunks; checkpoint is never content proof; lease precedes attempt | Already aligned as a rule; codify it and test it | C10 |
| System layer survey (`system-layer.html`) | Mantle stays a build tool (ADR 0010) | Reject the OS-scope parts; keep the introspection idea | — |

## 6. Candidate changes

Each candidate states a target, a home, the evidence that would prove it, and
its non-claims. Order is value first, risk second.

### C1. Retention interests replace one shared roots file

**Target.** Represent store retention as one assertion per owner:
`retain(owner, path, reason, declared-at)`. Persist each assertion as a record
whose filename is the canonical digest of its content, following the Synit user
settings pattern. Merge records deterministically into the GC root set.

**Why.** `state_dir/gc-roots.json` is a single mutable document. The operator CLI
already serializes its mutations, but one path has only one provenance slot;
it cannot represent separate owners' concurrent interests. Per-owner records
make release a retraction of one record, keep unrelated owners intact, and
give GC an auditable reason for every retained path.

**Home.** `src/store_cmd.rs` root commands, `crunch-gc-core` planning,
`crunch-store` persistence.

**Evidence.** A real two-process CLI fixture pins one signed path under two
different owner labels, rejects a foreign-label release, and leaves the second
record intact after releasing the first. A migration fixture archives the
legacy file and preserves `legacy-unmanaged` provenance; core and store tests
check plan equivalence and reject malformed, duplicate, oversized, and
unknown-version records. A missing PathInfo is rejected when pinning.

**Non-claims.** Owner labels are local declarations, not authenticated
credentials. Retention records prove declared interest only: not output
correctness, cache trust, or reachability.

### C2. `mantle watch`: re-evaluate on change and retract what disappears

**Target.** A watch mode that re-evaluates the selected Nickel source after a
change, diffs the resulting goal set against the live set, asserts new goals,
and retracts removed goals. Retraction cancels in-flight work for that goal.

**Why.** This is the Synit configuration watcher applied to a build plan. The
current loop re-runs a command and restarts selection from scratch. Watch mode
makes the developer loop incremental and makes stale work impossible: a goal
that no longer exists cannot keep building.

**Home.** CLI watch command, `crunch-pipeline` plan diff, `crunch-build`
scheduler cancellation.

**Evidence.** Positive: edit one root, only that root rebuilds; add a root, only
the new root dispatches. Negative: delete a root while it builds, the build is
cancelled and no output is recorded; a syntax error leaves the previous goal set
intact and reports the error; rapid edits coalesce without duplicate dispatch.

**Non-claims.** Watch mode is a developer convenience. It produces no release
evidence, and its live status is not a receipt.

### C3. Live build-state surface with subscription

**Target.** Publish bounded goal, worker, and resource facts to a coordination
surface that consumers subscribe to with patterns. Keep `--json build` and
`mantle-evaluation-stream-v1` unchanged as the compatibility surfaces.

**Why.** Onix, CI, and operator tools currently parse a one-shot stream or poll
reports. Subscription removes polling, and retraction removes stale "building"
state when a worker dies.

**Home.** Reuse the Molten (`aspen/`) dataspace component if its contract fits;
otherwise keep the projection inside Mantle behind a narrow port. Publish from
`crunch-build`; never let the projection own scheduling decisions.

**Evidence.** Positive: a subscriber sees discovery, dispatch, and terminal
facts in order. Negative: worker loss retracts its presence and its building
fact; a killed subscriber does not affect the build; the projection cannot
mutate store or scheduler state.

**Non-claims.** The surface is coordination state. Receipts and signed PathInfo
remain the only build evidence.

### C4. Causal build trace

**Target.** Emit `mantle-build-trace-v1` records with the Synit trace shape: one
record per scheduler action, with a stable action id, a `cause` field
(requirement, dependency-ready, dispatch, cache-decision, retry, cancellation),
and bounded action detail. Validate that every recorded action names a cause.

**Why.** Failure review currently reconstructs causality from logs. A causal
edge list answers "why did this build start?" and "what did this failure
release?" directly, and the trace schema is a reviewed vocabulary rather than a
new invention.

**Home.** `crunch-build` scheduler events, `src/transcript_cmd.rs`, build report
sidecar.

**Evidence.** Positive: a dependency failure produces a chain from root to leaf;
a cache hit names the admission check as its cause. Negative: an action without
a valid cause fails validation; a truncated trace fails closed; unknown action
kinds are rejected.

**Non-claims.** A trace proves the recorded causal order only. It does not prove
that the recorded order is complete or that any action succeeded.

### C5. One service-state vocabulary for long-lived Mantle components

**Target.** Adopt `started` / `ready` / `complete` / `failed` plus
user-defined states for the Rust cache daemon, remote serve, worker sessions,
and proof gates. Declare `depends-on` relationships. Name the restart policy
from the reviewed matrix: `always`, `on-error`, `all`, `never`. Make
`mantle doctor` publish derived state, not only print it.

**Why.** Three vocabularies currently describe readiness: goal states, daemon
startup sequences, and proof gates. One vocabulary makes readiness explicit and
testable, and dependency declarations replace implicit ordering.

**Home.** `crunch-rust-cache-core`, `src/remote_build*`, `src/operator_diagnostics.rs`,
bootstrap proof gates.

**Evidence.** Positive: a dependent service waits for `ready`, not for process
start. Negative: a process that exits before announcing readiness reports
`failed`, not `ready`; a blocked dependency never reports `complete`.

**Non-claims.** Declared state describes the component's own observation. It
does not prove output quality or proof correctness.

### C6. Declared demand for remote workers, with no capacity inference

**Target.** Emit an unsatisfied quantified requirement as a `require-worker`
assertion on the coordination surface. An operator-side supervisor may satisfy
it by starting a worker. Withdraw the assertion when the requirement is met,
cancelled, or satisfied another way.

**Why.** ADR 0025 correctly refuses to infer capacity from concurrency.
Declared demand is not inference: it states a requirement that already exists
in admitted facts, and it lets a supervisor respond without Mantle guessing.
The demand fact also makes capacity starvation visible instead of silent.

**Home.** `crunch-build::distributed` placement planning, remote farm
configuration, operator surface.

**Evidence.** Positive: a quantified job with no admitted worker publishes one
demand assertion; a worker that registers satisfies it and the assertion
retracts. Negative: demand never fabricates a lease; an unverified worker
presence cannot satisfy demand; duplicate demand deduplicates.

**Non-claims.** Demand states a requirement. It does not prove that capacity
exists, that a supervisor will respond, or that placement is optimal.

### C7. Attenuated authority for remote work, store views, and project scopes

**Target.** Add pattern caveats to the authority Mantle hands out: remote
builder tickets restricted to a declared job set, store views restricted to a
declared path pattern, and project-scoped configuration capabilities that
rewrite project declarations into project-namespaced goals.

**Why.** A restricted capability that carries its own restriction is simpler to
audit than a central policy re-check, and it composes: the holder can attenuate
further without consulting the issuer. The Synit rewrite/reject/alternative
semantics are a reviewed design for the filter language.

**Home.** `src/remote_credentials.rs`, `docs/store-authority-capabilities.md`
view construction, `mantle-project` scoping.

**Evidence.** Positive: a ticket restricted to one derivation refuses a second
derivation; caveat order composition is deterministic; an attenuation chain
verifies without the issuer online when the chain is verifiable. Negative: an
unknown caveat rejects everything; a rewritten assertion that fails its pattern
is dropped; attenuation cannot widen a grant.

**Non-claims.** Attenuation proves the restriction over the supplied filter
language. It does not prove the held build was correct.

**Prerequisite.** Coordinate with the stack's UCAN component and Basalt policy
boundary before adding any new token construction. If UCAN covers pattern
caveats, Mantle should consume it rather than define a second authority format.

### C8. Gatekeeper-shaped admission for remote entry points

**Target.** Give each remote entry point one resolve step that upgrades a
durable name to a live handle, backed by a table of bindings with generation
fences. Revocation retracts a binding.

**Why.** Mantle has several admission paths (tickets, substituters, base
layers, worker registration). One resolve step per entry point concentrates
admission and makes revocation uniform.

**Home.** `src/remote_build*`, `src/cache_substitution.rs`, base-store
selection in `crunch-store`.

**Evidence.** Positive: a valid name resolves once and yields a session handle;
a revoked binding refuses new sessions. Negative: an expired or forged name
fails closed; resolving the same name twice yields the same generation; a
revoked binding does not invalidate content that already passed admission.

**Non-claims.** A resolved handle proves admission at resolve time. It does not
prove later content identity; those checks remain separate.

**Prerequisite.** Only worth doing if the remote farm grows past the current
loopback and single-session paths.

### C9. Minimum-match patterns at declared extension points only

**Target.** Where a machine contract must accept additions from a newer
producer, accept extra fields and require the declared ones, and record which
bindings the match used. Keep exact decoding everywhere identity is computed.

**Why.** Synit group patterns match "at least" the declared structure. That
semantic is forward compatible. It is also dangerous near identity: an action
hash must not depend on which fields a pattern happened to read.

**Home.** `src/machine_contract_producer_tests.rs`,
`docs/nominal-dynamic-plan-types.md`, operator policy surfaces.

**Evidence.** Positive: a newer record with extra fields is admitted and the
used bindings are recorded. Negative: a missing required field is rejected; the
identity computation rejects the same record set regardless of extra fields.

**Non-claims.** Pattern admission proves shape acceptance. It does not prove
semantic compatibility with the newer producer.

### C10. Codify the transient-reference rule

**Target.** State and test the rule that a handle, session, or lease must be
introduced by a lifetime-bearing declaration before a transient message may use
it. Mantle already follows this for transfer sessions, checkpoints, and leases.

**Why.** The Synit membrane rules give the rule a precise form: a message must
not introduce an unknown reference, and an assertion must establish it first.
Mantle's "a checkpoint is never proof of content" and "a lease precedes an
attempt" are instances of it. Naming the rule makes new protocol work easier to
review.

**Home.** `docs/remote-transfer.md`, an ADR, plus a negative test per protocol
boundary.

**Evidence.** Positive: a message that references an established session
succeeds. Negative: a message that introduces an unknown session id fails
closed; a checkpoint without a manifest fails closed.

**Non-claims.** This is a protocol shape rule. It does not prove delivery,
completeness, or crash consistency.

## 7. What not to take

- **Do not rewrite the scheduler as an actor runtime.** Mantle is a batch,
  evidence-producing pipeline. Facets, turns, and linked tasks would add
  lifetime nondeterminism to a deterministic path. Take the semantics
  (retraction, lifetimes), not the runtime.
- **Do not replace Nickel with `.pr` scripts.** The manual itself calls the
  configuration scripting language inelegant and artificially limited. Typed
  Nickel contracts fit build configuration better.
- **Do not adopt the Syndicate wire protocol for content transfer.** Mantle's
  transfer plane has content identity, occurrence identity, quotas, and fences
  that the Syndicate protocol does not carry. Coordination facts only.
- **Do not build a dataspace inside Mantle.** Molten (`aspen/`) owns dataspaces
  in this stack. Reuse the published component or keep a narrow local port.
- **Do not make receipts or reports reactive.** The evidence plane stays
  canonical and replayable.
- **Do not infer capacity from concurrency or presence.** ADR 0025 already
  refuses this. Demand is declared; capacity is admitted.

## 8. Open questions

1. Does any current consumer (Onix lowering, CI, operator TUI) want a
   subscription surface, or is the NDJSON contract sufficient? C3 depends on
   this answer.
2. Does the stack's UCAN component already express pattern caveats over
   assertion-shaped payloads? If yes, C7 consumes it; if no, decide whether a
   bounded local caveat core is justified.
3. Is a developer watch mode in scope now, or does the fixed-point dev cache
   cover the current need? C2 stands or waits on that answer.
4. Which proof gates deserve milestone facts first? C5 needs one concrete
   consumer to avoid speculative infrastructure.

## 9. Change packages

The following Cairn change packages were created from these candidates under
`.cairn/changes/`. All tasks are open. Creating a package is not acceptance.

| Candidate | Change package | Delta spec | Plane |
|---|---|---|---|
| C1 | `add-retention-interest-records` | `store-lifecycle` | Building |
| C2 | `add-watch-mode-goal-retraction` | `build-scheduling` | Building |
| C3 | `publish-live-build-state-subscriptions` | `build-interchange` + `coordination-service` (new) | Build emits, daemon serves |
| C4 | `emit-causal-build-traces` | `operator-diagnostics` | Building |
| C5 | `unify-long-lived-service-readiness` | `service-readiness` (new) | Daemon and building |
| C6 | `declare-remote-worker-demand` | `build-scheduling` | Build derives, daemon serves |
| C7 | `attenuate-build-authority-with-pattern-caveats` | `build-authority-attenuation` (new) | Daemon issues, receivers enforce |
| C8 | `resolve-remote-entry-points-through-one-binding` | `remote-builds` | Daemon serves, receivers enforce |
| C10 | `codify-transient-handle-admission` | `remote-builds` | Building |

C9 (minimum-match pattern admission) has no package. It needs a named
consumer that an exact decoder breaks today. Without one, the change would be
speculative infrastructure.
