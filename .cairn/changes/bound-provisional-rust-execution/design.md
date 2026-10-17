# Design: Bound provisional Rust execution

## Context and current behavior

`execute_existing_units` in
`crates/mantle-rust-plan-app/src/application.rs:125-181` admits, restores or
executes, and finishes each effect before advancing. `ProducerArtifactObservation`
and `ResolvedUnitFacts` in `crates/mantle-rust-plan-core/src/plan.rs` already
bind producer, process, toolchain, arguments, environment, and artifact facts.
They do not authorize an in-flight producer to count as successful.

`crates/mantle-cargo-unit-plan/src/unit_helper.rs:80-97,486-504` requires an
existing linkable dependency and writes its manifest after rustc success.
`docs/shared-rust-unit-cache.md:91-121` already specifies complete immutable
object, signed envelope, then visibility marker publication. Those final-result
mechanisms remain authoritative. Generic `GoalState::Done` retains its meaning.

## Reference and scope

Reviewed 2026-10-15: Headstart
[`docs/design.md`](https://github.com/PowderworksCode/headstart/blob/ff81bfecda2116d54776a7fec538fc2f4e349d1f/docs/design.md)
and
[`docs/readiness.md`](https://github.com/PowderworksCode/headstart/blob/ff81bfecda2116d54776a7fec538fc2f4e349d1f/docs/readiness.md).
Borrow the confirmation concept, not its unbounded resident count or token-free
resume. No upstream code is imported by this proposal.

The session is Rust-owned and explicitly selected. Its first real consumer is
`pipeline-rust-interface-readiness`. An enabled session must execute real
admitted compilers; a fake process notification is not compilation evidence.

## Ownership and pure boundary

- **Core:** validate immutable session policy and acyclic producer-attempt
  facts; decide pending, provisional, confirmed, invalidated, and terminal
  dispositions; calculate admission/confirmation/rejection plans.
- **Application:** coordinate admitted process effects and apply observations
  through existing Rust execution/cache ports.
- **Shell:** own processes, private attempt roots, protocol decoding, enforced
  host resource controls, clocks, output retention, cancellation, and teardown.
- **Store/cache:** retain existing final-result verification and publication;
  do not expose provisional discovery or ambient session files.

Core decisions use canonical unit/attempt order and full BLAKE3 execution
identity, not notification arrival order. The shell records actual interleaving
for diagnostics without making timing part of durable action identity.

## Decisions

### Decision: Confirmation is distinct from process completion

**Choice:** A compiler's zero exit creates a completed provisional result if
any required producer attempt is unconfirmed. Confirmation requires the exact
producer-attempt closure to be successful. A different successful attempt for
the same unit cannot substitute silently. Invalidating an attempt invalidates
only its actual dependent closure.

**Rationale:** A dependent can finish before a producer's body error is known.
Its files and diagnostics cannot yet establish reusable success. Confirmation
happens before existing success receipt/cache publication hooks, not after them.
The dependency relation is acyclic and checked before effects begin.

### Decision: Private staging and bounded diagnostic quarantine

**Choice:** Stage artifacts and held per-unit diagnostics under a private,
exclusive session/attempt root. Release confirmed diagnostics in their original
per-unit order. Drop invalidated speculative diagnostics from normal compiler
output while retaining a redacted bounded reason in session diagnostics. Bound
both memory and any on-disk spool; output overflow is a typed session failure,
not truncation followed by a success claim.

**Rationale:** Existing complete-tree verification and no-clobber publication
already define final visibility. Early files neither bypass those checks nor
become signed PathInfo. Source/runtime inputs stay admitted and read-only.

### Decision: Resident processes retain reservations with notification-only rustc

**Choice:** Policy requires positive finite running, resident, memory, elapsed,
held-output, protocol-record, and teardown limits, plus a finite paused limit.
Resident processes include running and waiting compilers. Process counts cannot
exceed Mantle's existing in-flight ceiling of 64. The initial notification-only
adapter admits at most `max_running` resident compilers and retains each
execution reservation during a metadata wait. `max_paused` cannot exceed the
resident cap. A paused process becoming runnable therefore needs no new token
and cannot exceed the declared running ceiling.

**Rationale:** Headstart's `resume` notification describes a compiler that has
already resumed; accounting after receipt cannot enforce a strict CPU-slot
bound. Merely dropping a semaphore permit on `wait-metadata` is unsafe. Recycling
such slots requires an acknowledged pre-resume gate and a separately reviewed
protocol; it is not part of this change. The conservative adapter may leave a
slot idle, which is an explicit performance trade-off, not hidden oversubscription.

Use an admitted enforceable session memory boundary. RSS sampling is diagnostic
only and is not memory enforcement. Unsupported enforcement capabilities block
session enablement. Record the concrete checked policy profiles and mechanism
limits during implementation before exposing an enabled CLI/config path; there
is no ambient unlimited profile. Reserve capacity for producers before their
dependents; reject unknown/unadmitted producer attempts and validate low-slot
progress rather than relying on DAG acyclicity alone.

### Decision: Control is live; durable identity is complete

**Choice:** Decode bounded structured notifications with exact session, unit,
attempt, producer, and artifact identity. Unknown, stale, out-of-attempt, or
inconsistent transitions fail closed. Human log text is not the control plane.
Control frames must not wait behind held user diagnostics. Adapters without
live forwarding are not silently treated as compatible.

**Rationale:** `crates/crunch-rustc-wrapper/src/lib.rs:1218-1256` currently joins
bounded output capture after process completion; that is not live metadata
notification forwarding. Full source bodies, compiler bytes/revision, sysroot
and runtime closure, semantic options, environment/policy, and dependencies
remain action inputs. An interface hash cannot replace them.

### Decision: Failure retains ownership until teardown is observed

**Choice:** Producer failure/crash, cancellation, malformed protocol, and budget
exhaustion invalidate affected provisional work, stop further dispatch, cancel
owned descendants, and wait for observed process-tree termination and bounded
cleanup. A missed teardown deadline is a distinct incomplete-teardown failure;
it must not release an unproven reservation or claim successful cleanup.

**Rationale:** Dropping a future or seeing a lock disappear does not establish
that its child tree stopped. Stale files from an invalidated attempt are never
used by a later attempt.

## Alternatives considered

| Alternative | Disposition |
| --- | --- |
| Mark ordinary unit/store goals complete on early metadata | Rejected: weakens existing successful-output authority |
| Publish provisional results and delete them after failure | Rejected: a concurrent cache reader may already consume them |
| Release permits on wait and count resume afterward | Rejected: cannot enforce the declared bound |
| Preserve complete-result publication and retain reservations | Selected initial adapter; bounded, with possible idle capacity |
| Add generic cross-host speculative scheduling | Out of scope; no current transport/admission contract |

## Verification

Reuse `validate-rust-pipelining-equivalence` for real supported compiler runs and
exact private roots. Positive cases cover ordinary confirmation, multiple
producers, and independent roots. Negative cases cover upstream body failure,
process crash/cancellation, stale attempt notifications/artifacts, invalid
protocol order, held-output/record overflow, memory/elapsed exhaustion, and
unobserved teardown. Test one running slot and saturated limits with the actual
owned process boundary. Verify both absence of provisional visibility and
continued reuse of a separately confirmed result. Clock/resource observations
are recorded, never inferred from compiler exit or a mock echo.

## Risks and claim boundaries

Keeping reservations can reduce utilization; a later throughput policy requires
new evidence, not weakened bounds. Some hosts cannot enforce a session memory
boundary and must report blocked. Dropped speculative diagnostics do not justify
losing the original producer error. Session observations do not prove compiler
correctness, full Cargo compatibility, sandbox correctness, reproducibility,
performance improvement, or release eligibility. The host `rust-plan` lane
remains without ordinary derivation authority.
