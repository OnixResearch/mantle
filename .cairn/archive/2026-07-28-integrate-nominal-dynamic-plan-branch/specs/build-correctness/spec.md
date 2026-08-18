# Build Correctness Specification Delta

## ADDED Requirements

### Requirement: Reviewed nominal dynamic-plan integration is lossless

r[build_correctness.dynamic_plan_nominal.integration] Mantle MUST integrate a recorded immutable nominal dynamic-plan source commit without deleting unrelated target work or weakening admitted type boundaries.

#### Scenario: Integration binds exact source objects

r[build_correctness.dynamic_plan_nominal.integration.source]

GIVEN integration evidence records the source commit, tree, parent, subject, and changed-file set
WHEN Mantle starts the integration
THEN the actual Git objects MUST match the recorded source objects
AND a mutable branch name MUST NOT replace the immutable source identity.

#### Scenario: Integration starts from a clean preserved target

r[build_correctness.dynamic_plan_nominal.integration.target]

GIVEN main contains work that is not present in the source parent
WHEN an operator selects the integration target
THEN that work MUST have a committed preserved state
AND source application MUST occur in a clean dedicated integration worktree.

#### Scenario: Conflict resolution preserves both semantic sides

r[build_correctness.dynamic_plan_nominal.integration.preservation]

GIVEN a source file overlaps newer target work
WHEN the integration resolves that overlap
THEN the result MUST preserve unrelated target behavior and all reviewed nominal-domain behavior
AND it MUST NOT add raw-string fallback paths into admitted graph logic.

#### Scenario: Resolved target passes positive and negative checks

r[build_correctness.dynamic_plan_nominal.integration.validation]

GIVEN the source commit has been applied and all conflicts are resolved
WHEN focused and broader validation runs on the resolved tree
THEN valid plans MUST retain their accepted behavior
AND invalid wire values and compile-time role substitutions MUST still fail.

#### Scenario: Negative evidence rejects weakened boundaries

r[build_correctness.dynamic_plan_nominal.integration.validation.negative]

GIVEN a resolution exposes a nominal field, adds unrestricted conversion, erases a digest role, or bypasses wire admission
WHEN negative checks inspect or compile the resolved source
THEN the integration MUST fail before lifecycle closure.

#### Scenario: Canonical identity remains compatible

r[build_correctness.dynamic_plan_nominal.integration.compatibility]

GIVEN the accepted canonical `mantle-plan-v1` fixture
WHEN the resolved integration serializes and hashes the admitted plan
THEN its JSON bytes and BLAKE3 plan digest MUST match the reviewed source evidence.

#### Scenario: Policy checks retain zero targeted findings

r[build_correctness.dynamic_plan_nominal.integration.policy]

GIVEN the resolved core uses the declared Mantle nominal domains
WHEN Octet runs with the reviewed denial policy
THEN targeted primitive aliases, raw domain values, and invariant bypasses MUST remain absent.

#### Scenario: Integration evidence identifies the resolved tree

r[build_correctness.dynamic_plan_nominal.integration.evidence]

GIVEN all required validation has completed
WHEN Mantle records integration evidence
THEN the receipt MUST bind source commit, target commit, resolved tree, conflict decisions, checks, and bounded blockers.

#### Scenario: Lifecycle closes before main moves

r[build_correctness.dynamic_plan_nominal.integration.lifecycle]

GIVEN the integration tasks and gates pass
WHEN the integration branch is ready for operator review
THEN Mantle MUST sync and archive the integration change before its final integration commit
AND it MUST NOT push or move main without explicit operator instruction.
