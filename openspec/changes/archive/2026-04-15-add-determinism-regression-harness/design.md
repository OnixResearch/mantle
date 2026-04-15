# Design: Add determinism regression harness

## Context

A strong hermeticity story needs evidence, not just code shape. Once crunch has
explicit modes, reporting, envelope normalization, and stricter fallback rules,
it should keep them from regressing.

## Goals / Non-Goals

**Goals:**

- rerun representative builds under varied ambient host state
- compare output identity and hermeticity audit facts across runs
- keep the harness cheap enough for regular repo validation

**Non-Goals:**

- exhaustively prove bit-for-bit reproducibility of every package
- fuzz every builder or every bootstrap stage

## Decisions

### 1. Compare both output digests and audit facts

**Choice:** the harness compares final output digest and emitted hermeticity
audit facts, not just exit status.

**Rationale:** a build can still succeed while losing hermeticity strength.

### 2. Cover three representative paths

**Choice:** the first harness version covers:

- one normal derivation,
- one fetcher-rooted build,
- one self-build-friendly path.

**Rationale:** those paths exercise different impurity risks without making the
initial harness too large.

## Risks / Trade-offs

**Longer CI or local validation time**
Repeated builds cost time. The harness should start with a small matrix and grow
only when it finds real bugs.
