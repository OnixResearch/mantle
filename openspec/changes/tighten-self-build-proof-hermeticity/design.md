# Design: Tighten self-build proof hermeticity

## Context

Crunch's self-hosting proof already records tool provenance, but later stages
still need a stronger policy boundary than stage0. Once crunch has built its own
`busybox` and `bwrap`, reusing host fallback discovery weakens the proof claim.

## Goals / Non-Goals

**Goals:**

- let strict mode apply to self-build and proof flows
- require exact crunch-built tool roots after the bootstrap-tool step
- report later-stage fallback behavior explicitly

**Non-Goals:**

- remove stage0 host prerequisites entirely
- redesign the whole proof workflow

## Decisions

### 1. Stage0 and later stages have different trust rules

**Choice:** stage0 may still use declared host prerequisites, but stage1 and
later stages must reject host fallback tool rediscovery in strict mode once
crunch-built tool roots exist.

**Rationale:** this matches the real bootstrap boundary and strengthens the
later-stage proof claim without pretending stage0 is already impurity-free.

### 2. Proof output must name exact tool roots

**Choice:** proof summaries report the exact `busybox` and `bwrap` roots used by
later stages and whether any fallback event occurred.

**Rationale:** proof consumers need concrete provenance, not just a green test.

## Risks / Trade-offs

**Strict proof mode may fail on hosts that still pass practical mode**
That is expected. The stricter mode exists to catch exactly those weaker paths.
