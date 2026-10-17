# ADR 0089: Bind static inputs to dynamic-plan roots by request identity

## Status

Proposed (2026-09-30). Implementation and focused tests are complete.
A real one-run CLI fixture exercises input-addressed and content-addressed
plan roots, a two-reference consumer, unchanged cached rebuilds, typed
failure controls, and persisted unsigned provenance. Isolated spec
sync/archive and acceptance remain open.

## Context

A native dynamic plan can produce and register root units only after its
producer builds. Static Nickel derivations currently can depend on the
producer's plan artifact, but cannot reference a root unit's output. Requiring
a second evaluation or wrapper per root would break one-run composition.
A consumer needs a stable derivation identity before the producer runs,
including when the referenced root's output is content-addressed and has no
realized store path yet.

## Decision

Introduce the typed Nickel input `{ name, producer, plan_output, root,
unit_output }`, where `producer` is an inline derivation and `plan_output`
must belong to its declared `dynamic_plan_outputs`. `root` names an exported
plan root unit, not an internal unit; `name` identifies the deterministic
`{{mantle-plan-output:<name>}}` marker in consumer arguments/environment.
The selected plan output is an ordinary input derivation edge. An additional
compact, name-sorted JSON binding record under the reserved
`__MANTLE_PLAN_OUTPUT_BINDINGS` environment key covers the producer
*derivation path*, plan output, root unit id, unit output name, local input
name, and deterministic placeholder. Together that edge and record bind the
consumer's request identity and its input-addressed output paths at
evaluation. Derivations without the input keep their prior bytes and paths.
No ATerm or derivation-format field changes.

The conversion layer validates names, the declared plan output, and at most
16 distinct references. A reference marker without a matching input fails
evaluation. After a producer succeeds, the worker binds references against
an *accepted* plan, checking exported root membership before output
membership, and installs waits on those root goals before the consumer may
dispatch. A producer failure or rejected plan fails bound consumers without
a successful output. Once the root completes, the worker substitutes its
realized output path for the placeholder, mounts that path with its closure,
adds it to reference-scan needles, and reports both the symbolic request
and realized binding. The first failing reference in canonical input-name
order determines the recorded reason; no unresolved placeholder may enter a
sandbox.

## Consequences and non-claims

The path of an input-addressed consumer depends on the producer derivation
request, not on an observed plan digest; reports include that digest for
review. A nondeterministic producer can still bind different content under
one request identity, as with ordinary input-addressed builds. This
mechanism does not implement evaluator suspension, import-from-derivation,
Nix `builtins.outputOf` wire compatibility, binding of non-root units, or
resolved content-addressed identity for the consumer. A successful
binding/report proves the observed graph association and path substitution,
not producer determinism, root correctness, consumer correctness, or release
eligibility.

For a successful bound consumer, the worker also records the accepted plan
digest and realized root derivation/output paths alongside the declared
producer derivation path, plan output, root id, and unit output in both the
build report and the canonical artifact provenance
`Claims.extra["mantle.plan_output_bindings"]`. These are observed binding
facts, not a producer-authored correctness assertion. The persisted
provenance has a canonical BLAKE3 digest but **no cryptographic signature**;
the signed store PathInfo does **not** bind its attestation digest. Neither
canonicalization nor the PathInfo signature makes these observed facts a
signed provenance claim.
