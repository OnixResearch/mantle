# Design: Cross-run resume and runtime confirmation

## Context

Attempt-local markers can support recovery in one staging directory. Fresh-directory resume also needs the stage artifacts and execution-tree state that later stages consume.

The resume boundary must distrust marker status. It must remeasure all identities before the shell restores state or skips work.

## Decisions

### Decision: Build a pure resume plan from remeasured facts

The core receives current source, plan, policy, stage, producer, and output identities. It returns an ordered restore plan or a fail-closed restart decision.

The core does not read files or trust producer status fields.

### Decision: Publish one content-addressed bundle per completed stage

The shell stages the required execution tree, outputs, and marker facts. It publishes the bundle only after the core accepts the complete identity set.

A partial, stale, mismatched, or unknown bundle cannot authorize a skip.

### Decision: Restore into a fresh staging directory

A dev resume creates a fresh staging directory, revalidates the selected bundle, restores the required state, and continues at the first incomplete stage.

The report records restored stages separately from executed stages.

### Decision: Keep promoted proof authority cold

A promoted run ignores all resume bundles and dev stores. It starts from empty provider and output authority and cannot publish success from restored work.

## Validation

Positive coverage includes a fresh-directory transition resume and a complete cold-to-cached-to-adopt dev cycle.

Negative coverage includes modified markers, missing execution-tree members, stale source, wrong producer, wrong policy, unknown stage, partial publication, and promoted-path cache use.

## Non-Claims

- A restored stage was not executed again in the resumed attempt.
- A dev adoption cycle does not satisfy a promoted fixed-point proof.
- Runtime confirmation remains bounded to the recorded source profile, host, plan, and policy.
