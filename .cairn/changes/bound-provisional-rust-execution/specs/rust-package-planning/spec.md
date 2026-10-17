# Specification: Bounded provisional Rust sessions

## ADDED Requirements

### Requirement: Session admission binds exact producer attempts

r[rust_package_planning.provisional_session.admission] Mantle MUST admit an
explicitly selected Rust execution session from bounded immutable policy,
acyclic producer dependencies, full unit identities, and exact attempt
identities before dispatch. The default execution paths MUST remain unchanged.
The Rust core MUST make decisions over supplied facts without process, clock,
filesystem, environment, or network effects.

#### Scenario: Declared ordinary compiler session is admitted

- GIVEN a supported Rust topology, complete declared process inputs, and an enforceable finite session policy
- WHEN an operator explicitly selects the bounded session
- THEN each dispatched compiler MUST bind its unit, attempt, and required producer attempts
- AND ordinary host execution MUST NOT acquire sandbox derivation or PathInfo authority

#### Scenario: Invalid producer or unavailable bound is rejected

- GIVEN a cycle, unknown producer attempt, missing limit, or unavailable required memory-enforcement capability
- WHEN the session is admitted
- THEN Mantle MUST return a typed rejected or blocked disposition before compiler dispatch
- AND MUST NOT replace the missing capability with an unbounded or ambient fallback

### Requirement: Confirmation precedes reusable success

r[rust_package_planning.provisional_session.confirmation] A completed compiler
whose required producer attempts are not all confirmed MUST remain provisional.
Mantle MUST keep its artifacts and user-facing diagnostics private until its own
success and its exact producer-attempt closure permit confirmation. Existing
complete-result verification and publication MUST remain in force. Early
metadata MUST NOT satisfy ordinary completed store-output or goal semantics.

#### Scenario: Producer success releases a completed dependent

- GIVEN a dependent compiler has succeeded while an exact required producer remains unconfirmed
- WHEN that producer and all other required attempts succeed and are confirmed
- THEN the dependent MAY be confirmed through existing final-result checks
- AND its retained diagnostics MUST be released in original per-unit order before reporting its final result

#### Scenario: Producer fails after a dependent has finished

- GIVEN a completed provisional dependent and its private artifacts
- WHEN an exact required producer fails
- THEN no success receipt, local/shared cache candidate, signed PathInfo, or final unit manifest for the affected dependent MUST become visible
- AND speculative dependent diagnostics MUST NOT replace or obscure the original producer error

#### Scenario: Another attempt cannot substitute for the failed producer

- GIVEN a dependency has a successful older or later attempt with the same unit identity
- WHEN the dependent names a different unconfirmed or failed producer attempt
- THEN that other attempt MUST NOT silently satisfy confirmation

### Requirement: Paused processes remain within enforced session bounds

r[rust_package_planning.provisional_session.resource_bounds] Policy MUST impose
finite running, resident, paused, held-output, protocol-record, session-memory,
elapsed-time, and teardown limits. Resident count MUST include paused compilers
and MUST NOT exceed the existing 64-process in-flight ceiling. The initial
notification-only adapter MUST retain execution reservations during waits and
cap resident compilers at the configured running limit; observing `resume`
after execution continues MUST NOT be treated as pre-resume permission. Memory
sampling MUST NOT be presented as enforcement. Resource clocks and observations
MUST remain outside durable action identity.

#### Scenario: Several paused compilers become runnable

- GIVEN a notification-only session whose resident processes all retain reservations
- WHEN multiple waiting compilers resume together
- THEN the declared running and resident ceilings MUST remain satisfied
- AND no compiler MUST borrow an undeclared extra jobserver token or slot

#### Scenario: Budget is exhausted

- GIVEN held diagnostics, protocol records, process counts, memory, or elapsed time reach the admitted limit
- WHEN additional work would violate the limit
- THEN Mantle MUST stop admission or fail the affected session with the exact typed budget disposition
- AND MUST NOT truncate required output and report successful confirmation

#### Scenario: A producer needs progress under a low slot limit

- GIVEN an admitted dependency chain and one running slot, or a fully occupied session
- WHEN dependent work would otherwise consume capacity needed by its producer
- THEN admission MUST preserve producer progress without exceeding configured bounds
- AND the actual process scenario MUST terminate within the session deadline rather than rely only on an acyclicity argument

### Requirement: Failure retains ownership through observed teardown

r[rust_package_planning.provisional_session.failure_teardown] Producer failure,
crash, cancellation, invalid control, or budget exhaustion MUST invalidate the
actual affected dependent-attempt closure and prevent its confirmation. The
shell MUST own cancellation and process-tree teardown until termination is
observed. Expired teardown MUST be an explicit incomplete-teardown failure,
not a successful cleanup or an unproven reservation release. Independent
confirmed work MUST NOT be invalidated by an unrelated failure.

#### Scenario: Producer crashes while dependents wait

- GIVEN dependent compiler processes await an exact producer artifact
- WHEN the producer crashes without writing the artifact
- THEN affected dependent attempts MUST terminate or be cancelled under the admitted deadline
- AND stale files MUST NOT make a later attempt appear successful

#### Scenario: Cleanup cannot be observed

- GIVEN cancellation was requested but a child tree has not been observed terminated
- WHEN the teardown deadline expires
- THEN Mantle MUST report incomplete teardown and retain the unproven ownership/reservation state
- AND MUST NOT claim that dropping a future or releasing a file lock proved cleanup

#### Scenario: Independent confirmed result survives

- GIVEN two independent roots and one confirmed reusable result
- WHEN the other root's producer fails
- THEN the confirmed independent result MUST retain its existing admission and reuse disposition

### Requirement: Live control and complete identity remain separate from held output

r[rust_package_planning.provisional_session.control_identity] Mantle MUST process
bounded structured control notifications independently from held diagnostics and
bind them to the exact session, unit, attempt, producer, and artifact. Unknown,
stale, inconsistent, or unsupported control MUST fail closed. Supported wrapper
integration MUST forward live control or explicitly reject the session. Full
source bodies, compiler, sysroot/runtime closure, semantic options, admitted
environment, policy, and dependencies MUST remain execution identity inputs;
interface bytes alone MUST NOT authorize final reuse.

#### Scenario: Diagnostics are held but control remains live

- GIVEN a provisional compiler emits both diagnostics and a supported artifact or wait notification
- WHEN its diagnostics are quarantined
- THEN the control notification MUST be processed without waiting for producer confirmation or compiler exit
- AND its artifact readiness MUST NOT be promoted to final success

#### Scenario: Buffering wrapper or stale notification is selected

- GIVEN a wrapper without live forwarding, or a notification naming another attempt
- WHEN the experimental session evaluates the integration
- THEN Mantle MUST report an explicit incompatibility or protocol failure
- AND MUST NOT infer readiness from a human log or a similarly named file

#### Scenario: Body change preserves the visible interface

- GIVEN identical interface bytes but different source bodies or semantic compiler options
- WHEN execution and reuse identities are computed
- THEN the complete admitted identities MUST distinguish those inputs
- AND an interface-only cache lookup MUST NOT bypass current result verification
