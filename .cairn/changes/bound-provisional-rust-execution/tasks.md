# Tasks: Bound provisional Rust execution

Proposed implementation work. No item below is completed by authoring this
package or by structural Cairn validation.

Implementation admission rechecked 2026-10-23: blocked, not accepted. The
selected compiler rejects the required early-metadata option; the pinned
reference is not a source-fixed qualified provider, and the current user has
no delegated finite cgroup session boundary. No provisional session runtime
or tests were delivered. See the [fresh probes and success/publication hook
map](../pipeline-rust-interface-readiness/evidence/authoring-validation.md#current-implementation-admission-inspection-2026-10-23)
and [proposed ADR 0126](../../../adr/0126-require-qualified-providers-before-admitting-bounded-rust-sessions.md).
Existing serial execution is not completion of any provisional-session task.

## Phase 1: Contract and existing boundaries

- [ ] [serial] T1.1 Capture the current successful/failing serial Rust execution and cache-publication behavior using the differential baseline, and map every success/publication hook that the session must gate. r[rust_package_planning.provisional_session.confirmation]
- [ ] [serial] T1.2 Define pure session/attempt facts, canonical ordering, acyclic producer admission, state transitions, and typed failure dispositions using the existing Rust planning core/application ports. r[rust_package_planning.provisional_session.admission]
- [ ] [serial] T1.3 Select and validate finite Nickel-authored session profiles, the enforceable host memory boundary, and explicit unsupported-capability behavior; record the reservation/confirmation design in an ADR before enabling execution. r[rust_package_planning.provisional_session.resource_bounds]

## Phase 2: Session implementation

- [ ] [serial] T2.1 Implement private exclusive attempt staging, exact producer-attempt binding, and confirmation before success receipts, local/shared cache visibility, or final artifact publication; preserve existing final-result verification. r[rust_package_planning.provisional_session.confirmation]
- [ ] [serial] T2.2 Implement resident/running/paused and held-output/protocol limits; retain execution reservations across notification-only compiler waits and prevent stale attempt re-use. r[rust_package_planning.provisional_session.resource_bounds]
- [ ] [serial] T2.3 Implement bounded live control decoding separate from held user diagnostics, complete action identity binding, and explicit rejection of non-forwarding wrappers and malformed transitions. r[rust_package_planning.provisional_session.control_identity]
- [ ] [serial] T2.4 Implement dependent invalidation, cancellation, process-tree ownership, teardown deadlines, and incomplete-teardown outcomes without releasing unproven reservations. r[rust_package_planning.provisional_session.failure_teardown]

## Phase 3: Behavioral acceptance

- [ ] [parallel] T3.1 Exercise real successful compiler units with multiple producers and independent roots; prove original per-unit diagnostic order, final cache visibility after confirmation, and reuse of a separately confirmed result. r[rust_package_planning.provisional_session.confirmation]
- [ ] [parallel] T3.2 Exercise producer failure/crash, cancellation, stale final output, wrong-attempt notification, and protocol rejection; prove no affected success receipt/cache marker is visible and observe owned child-tree termination. r[rust_package_planning.provisional_session.failure_teardown]
- [ ] [parallel] T3.3 Exercise one-slot and saturated-session progress, paused-to-running transitions, every configured count/byte/memory/deadline boundary, and an unsupported enforcement host; retain exact limit and teardown outcomes. r[rust_package_planning.provisional_session.resource_bounds]
- [ ] [parallel] T3.4 Change source bodies, compiler/sysroot/options/policy, and dependency attempt identity while interfaces remain unchanged; prove no interface-only cache alias and no host-output promotion into PathInfo. r[rust_package_planning.provisional_session.control_identity]

## Phase 4: Verification and acceptance

- [ ] [serial] T4.1 Run the supported differential matrix and real session smoke, focused first-party formatting/Clippy/tests and no-std boundary checks; capture exact failures and successful observations, not skipped experimental cells. r[rust_package_planning.provisional_session.admission]
- [ ] [serial] T4.2 Update operator/cache documentation and compatibility declarations, run Cairn validate plus proposal/design/tasks gates, and retain command transcripts with their exact input identities. r[rust_package_planning.provisional_session.control_identity]
- [ ] [serial] T4.3 Obtain implementation and policy acceptance; commit implementation before any separately authorized sync/archive and preserve final post-archive validation if archiving is requested. r[rust_package_planning.provisional_session.failure_teardown]
