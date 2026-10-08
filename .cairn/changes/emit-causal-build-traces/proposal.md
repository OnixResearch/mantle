# Proposal: Emit causal build traces

## Why

Failure review reconstructs causality from build logs and receipts. A receipt
records what happened, not why it happened. The build log interleaves unrelated
events, so a dependency failure needs manual reading to connect the root
requirement to the failing leaf.

The Synit manual records one trace entry per actor activation with an explicit
`cause` field and a bounded action taxonomy
(`~/.local/share/mantle-references/synit-book/pages/42-protocols__syndicate__trace.md`,
reviewed in `docs/synit-application-notes.md`). Mantle already has local
goal transitions, dispatch, and cache decisions to observe. Ordinary local
`mantle build --causal-trace` has no scheduler retry or cancellation transition;
watch-mode cancellation is a separate, untraced flow. Diagnostic emission must
not invent either event.

A causal trace is a diagnostic artifact. It does not replace receipts and does
not become build evidence.

ADR 0080 keeps trace emission in the building plane. The coordination daemon
may serve traces to subscribers.

## What Changes

- Emit one bounded `mantle-build-trace-v1` record per scheduler action with a
  stable action identity, the goal identity, a bounded action kind, and the
  causing action's identity.
  r[mantle.operator_diagnostics.causal_trace_records]
- Name a closed cause vocabulary: root requirement, dependency ready, dispatch,
  cache decision, retry, cancellation, cleanup, and external trigger. Every
  record MUST name a cause from this vocabulary.
  r[mantle.operator_diagnostics.causal_trace_cause_validation]
- Validate the trace on read: an unknown cause, a missing cause, or a cause
  that names an absent action MUST fail closed.
  r[mantle.operator_diagnostics.causal_trace_cause_validation]
- Bound and redact the trace: declared record, byte, and action counts;
  absolute paths, credential-shaped values, and standalone digests redacted
  under the existing diagnostic rules.
  r[mantle.operator_diagnostics.causal_trace_bounds]
- Keep the trace opt-in. Existing reports, receipts, and logs keep their
  current shape.

## Impact

- **Immediate consumer**: failure triage for local builds and the operator
  review of long proof runs.
- **Immediate outcome**: a failing leaf can be traced to its root requirement
  without log reading.
- **Durable capability**: a causal vocabulary that later serves remote
  attempt review and CI failure summaries.
- **Maintenance owner**: Mantle operator diagnostics owner.
- **Repeatability evidence**: a dependency-failure chain fixture, a cache-hit
  cause fixture, a retry fixture, and negative fixtures for unknown, missing,
  and dangling causes.
- **Compatibility**: the trace is a new opt-in artifact.

## Scope

The change covers the trace record schema, the cause vocabulary, emission from
scheduler actions, read-time validation, bounds, redaction, and fixtures.

## Out of Scope

- Replacing the build log, the transcript, or the remote trace context.
- Cross-host trace correlation.
- Treating a trace as build evidence, release evidence, or proof input.
- Proving that the recorded order is complete.

## Success Criteria

- A dependency failure yields a chain from root requirement to failing leaf.
- A cache hit names its admission check as cause.
- An unknown, missing, or dangling cause fails validation.
- Trace bounds and redaction match the existing diagnostic contract.
