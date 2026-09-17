# Specification: Watch mode plan assertion and retraction

## ADDED Requirements

### Requirement: Watch mode diffs goal sets by identity

r[build_scheduling.watch_plan_assertion] An opt-in watch mode MUST re-evaluate
the selected source after a change and compute the admitted goal set for the
current source. It MUST diff that set against the live goal set by goal
identity and classify every goal as added, retained, or retracted.

Retained goals MUST keep their completed work. Added goals MUST be dispatched
under the existing scheduling rules. Watch transitions MUST be reported as
bounded events with a stable run identity.

#### Scenario: One-root edit rebuilds one root

- GIVEN a watch session with two admitted roots and both goals complete
- WHEN the source changes one root's definition
- THEN the diff MUST mark that goal retracted and its replacement added
- AND the unrelated root MUST remain retained without a rebuild

### Requirement: Retraction cancels in-flight work

r[build_scheduling.watch_retraction_cancellation] When the diff retracts a goal,
the mode MUST request cancellation of its in-flight build and release its
reservation. A retracted goal MUST NOT be recorded as a successful output. A
build that does not stop within the declared cancellation window MUST be
reported as pending cancellation rather than silently retained.

#### Scenario: Deleted root stops building

- GIVEN a watch session with a root whose build is in flight
- WHEN the source change removes that root
- THEN the mode MUST retract the goal, request cancellation, and release its
  reservation
- AND the cancelled goal MUST NOT be recorded as a successful output

#### Scenario: Cancellation is bounded

- GIVEN a retracted goal whose build does not stop within the declared
  cancellation window
- WHEN the mode reports the retraction
- THEN it MUST report pending cancellation for that goal
- AND it MUST NOT report the goal as retained

### Requirement: Failed re-evaluation retains the admitted goal set

r[build_scheduling.watch_error_retention] An evaluation, conversion, or policy
failure during re-evaluation MUST leave the previously admitted goal set
running. The mode MUST report the error and MUST NOT retract goals because of a
failed re-evaluation. Live goal counts, event counts, and re-evaluation rate
MUST be bounded, and a bound violation MUST fail closed.

#### Scenario: Syntax error preserves running goals

- GIVEN a watch session with running or completed goals
- WHEN the edited source fails evaluation or conversion
- THEN the session MUST keep the previously admitted goal set
- AND it MUST report the error without cancelling those goals

#### Scenario: Edit coalescing

- GIVEN several source changes within one re-evaluation window
- WHEN the mode re-evaluates once for the coalesced change
- THEN it MUST dispatch each added goal at most once
