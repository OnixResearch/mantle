# Design: Cross-run resume and runtime confirmation

## Context

Attempt-local markers can support recovery in one staging directory. Fresh-directory resume also needs the stage artifacts and execution-tree state that later stages consume.

The resume boundary must distrust marker status. It must remeasure all identities before the shell restores state or skips work.

## Decisions

### Decision: Build a pure resume plan from remeasured facts

`crunch-dev-resume-core` is a `no_std + alloc` functional core. It receives current source, plan, policy, stage, producer, output, payload, and bundle identities.

The core returns an ordered restore plan or a fail-closed restart decision. It does not read files or trust producer status fields.

### Decision: Publish one content-addressed bundle per completed stage

The shell publishes one manifest for each of the six ordered stages. Shared immutable payload objects use BLAKE3 identities, so stage manifests do not duplicate large trees.

The manifest becomes visible only after every referenced object passes remeasurement. A partial, stale, mismatched, conflicting, or unknown bundle cannot authorize a skip.

### Decision: Reuse bounded checkpoint I/O without sharing authority

The dev shell reuses the existing no-follow observation, copy, and no-replace publication mechanisms. Dev manifests and promoted checkpoint manifests keep separate origins and cache namespaces.

### Decision: Restore into a fresh staging directory

A dev resume creates a fresh staging directory. The shell remeasures the selected objects, restores the required state, and continues at the first incomplete stage.

The machine report records restored stages separately from stages executed in the current attempt.

### Decision: Keep promoted proof authority cold

A promoted run rejects dev resume options before cache access. It starts from empty provider and output authority and cannot publish success from restored work.

## Validation

Positive coverage includes a fresh-directory transition resume and a complete cold-to-cached-to-adopt dev cycle.

Negative coverage includes modified markers, missing execution-tree members, stale source, wrong producer, wrong policy, unknown stage, partial publication, and promoted-path cache use.

## Non-Claims

- A restored stage was not executed again in the resumed attempt.
- A dev adoption cycle does not satisfy a promoted fixed-point proof.
- Runtime confirmation remains bounded to the recorded source profile, host, plan, and policy.
