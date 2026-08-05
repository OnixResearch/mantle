# Reject Picolibc for the StageX C runtime

## Status

Accepted (2026-08-04)

## Context

The StageX protected path builds a bounded native-musl runtime from
authenticated musl 1.1.24 source: 746 self-host sources and 19 predecessor
sources (765 compiled units), 106 recorded source-modification operations,
and a passing behavior contract.

Picolibc 1.8.12 ships an x86_64 Linux static profile that could have reduced
the early C runtime surface. The `evaluate-picolibc-stagex-runtime` change
built that profile as a research-only diagnostic under pinned source and
signed toolchain authority, ran the shared StageX libc behavior contract
against it, and compared it with the native-musl baseline through a pure
decision core.

## Decision

The comparison outcome is `rejected`. Provider selection, StageX lineage,
bootstrap parity, accepted provider digests, and release status remain
unchanged. Picolibc source and build tooling stay outside the protected
StageX execution graph.

Two independent reasons forced the outcome:

1. **Behavior failure.** Picolibc fails the shared positive behavior
   contract at the invalid-signal case: `sigaction(0, ...)` must return -1
   with `EINVAL`, and Picolibc's range check (`sig < 0 || sig >= _NSIG`)
   accepts signal 0 where musl rejects it. The malformed-source rejection
   case passed.
2. **No surface reduction.** The diagnostic compiles 1,222 translation units
   against the baseline's 765. The candidate bar requires fewer compiled
   units than the baseline.

Both isolated builds (fresh state, output, and scratch roots) produced
byte-identical output trees, attestation BLAKE3
`f0cc10766a6222c76753317d0c35fb3bcb973e9d44e8b999375bdf439fea4317`, so the
construction itself is deterministic. The rejection is about suitability,
not repeatability.

The comparison report lives at
`cairn/archive/2026-08-04-evaluate-picolibc-stagex-runtime/evidence/comparison-report.json`
with the full fact set, and the oracle checkpoint beside it names the
question, evidence, decision, owner, and next action.

## Alternatives Considered

### Patch Picolibc and rerun the comparison

Rejected. The behavior gap is one `sigaction` range check, but the
unit-count gap is structural: 1,222 versus 765 units cannot be patched into
a reduction. A patched fork would also break the pinned upstream source
authority this comparison relied on.

### Count only the Linux profile objects

Rejected. The comparison counts what the build compiles, because compile
load is the surface the early runtime pays for. Rescoping the count after
seeing the result would be narrative, not evidence.

### Treat the rejection as Picolibc unsuitability everywhere

Rejected. This decision is scoped to the StageX early C runtime comparison
against the current native-musl boundary. It says nothing about Picolibc on
embedded targets, other architectures, or other consumers.

## Non-claims

This ADR does not claim that Picolibc is incorrect, that musl is correct,
that the compiler or kernel is correct, or that any provider admission
state changed. A future Picolibc release that passes the shared behavior
contract with fewer compiled units can re-enter through a new Cairn change
with fresh construction and admission evidence.
