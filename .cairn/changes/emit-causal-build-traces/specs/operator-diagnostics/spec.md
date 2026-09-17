# Specification: Causal build traces

## ADDED Requirements

### Requirement: Causal trace records name their cause

r[mantle.operator_diagnostics.causal_trace_records] Mantle MUST be able to emit
a bounded `mantle-build-trace-v1` record for each scheduler action. Each record
MUST carry a stable action identity, the goal identity when one applies, a
bounded action kind, and the identity of the causing action.

Emission MUST be opt-in, and existing reports, receipts, logs, and transcripts
MUST keep their current shape.

#### Scenario: Dependency failure chains to its root

- GIVEN a root goal that depends on a failing leaf derivation
- WHEN the trace is collected
- THEN the failing action MUST be reachable from the root requirement by
  cause identities

#### Scenario: Cache decision names its cause

- GIVEN a goal satisfied from cache
- WHEN the trace is collected
- THEN the record for the reuse MUST name the admission check as its cause

### Requirement: Cause validation fails closed

r[mantle.operator_diagnostics.causal_trace_cause_validation] Trace reading MUST
validate every record against a closed cause vocabulary. An unknown cause, a
missing cause, or a cause that names an absent action MUST fail closed. Trace
record count, byte size, and action count MUST be bounded, and a bound
violation MUST fail closed. Record content MUST be redacted under the existing
diagnostic rules.

#### Scenario: Unknown cause rejected

- GIVEN a trace record whose cause is not in the declared vocabulary
- WHEN the trace is read
- THEN validation MUST reject the trace

#### Scenario: Dangling cause rejected

- GIVEN a trace record whose cause names an action absent from the trace
- WHEN the trace is read
- THEN validation MUST reject the trace

### Requirement: Traces stay outside the evidence plane

r[mantle.operator_diagnostics.causal_trace_bounds] Trace records MUST NOT appear
in receipts, attestations, or release evidence, and MUST be rejected by the
existing evidence validators. Enabling tracing MUST NOT change build outputs or
receipt bytes.

#### Scenario: Receipts unchanged with tracing enabled

- GIVEN the same build inputs and policy
- WHEN one build runs with tracing enabled and one without
- THEN the recorded receipts MUST be byte-identical
- AND the evidence validators MUST reject the trace file
