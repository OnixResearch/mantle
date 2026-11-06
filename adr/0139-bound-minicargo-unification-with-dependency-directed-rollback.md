# ADR 0139: Bound minicargo unification with dependency-directed rollback

## Status

Accepted (2026-11-06) for source-program preparation. This decision does not
accept a source-built provider, a fixed point, or a release.

## Context

The Rust 1.90 source author consumes the mrustc 0.12 package-selection policy.
Its original Cargo.lock reader is disabled. Mantle must resolve the authenticated
vendored manifests, including compatible-version unification and exact pins such
as rustc_log's tracing-core dependency, before authoring inert build programs.

Chronological recursive search revisits unrelated version choices after a late
conflict. Filtering incompatible versions alone is insufficient: an empty
filtered set is a unification conflict, not an absent optional vendor package.
The real preparation reached sustained search without completing; a later
filtered implementation also continued searching without a result.

## Decision Drivers

- Preserve source-selected requirements, highest-first candidates, existing
  satisfying-version reuse, and separate incompatible version families.
- Never drop a required edge to disguise an unresolved version conflict.
- Skip choices that cannot affect a conflict without losing viable solutions.
- Bound work and avoid native call-stack depth proportional to package count.
- Keep the current consumer and maintenance owner in Mantle's Rust source author.

## Decision

Use an explicit decision stack and record the choices that introduced each
resolved manifest. Path dependencies inherit their introducing choices. Forced
selections also retain the choices that excluded other candidates. On conflict,
roll back directly to an involved decision; exhausted alternatives propagate
their accumulated causes, including the requiring package and filtered choices.
Snapshots restore the queue, visited manifests, resolved versions, provenance,
and selected edges together. Only genuine multi-candidate choices need snapshots;
the last alternative consumes its saved state.

Index vendor paths by package name once per resolution. Retain mrustc requirement
comparison, deterministic highest-first ordering, and reuse of an already
satisfying version. Truly absent vendor versions remain soft failures during
selection; activation still rejects an enabled missing dependency. An empty
filtered set is never treated as absence.

Fail closed after 65,536 dependency-processing steps or 4,096 simultaneous
choice frames. Bound exhaustion is not a partial solution or an unsatisfiability
proof. Do not use Cargo.lock as a selection oracle, prune lower patch versions,
or fall back to a greedy answer. The limits bound this implementation's search;
they do not make dependency resolution polynomial or prove mrustc equivalence.

## Consequences

The current consumer gains repeatable, finite-work preparation rather than an
unbounded chronological search. Mantle's Rust source-program author owns the
implementation and regression fixtures. A late exact-pin fixture with 24
independent binary choices must resolve within 128 dependency steps. Exhaustive
three-package constraint fixtures compare satisfiability and returned selections
against all version assignments. Further fixtures cover requiring-package
rollback, path provenance, filtered alternatives, incompatible families,
contradictory pins, absent dependencies, and work-bound failure.

Real preparation remains the integration check. Successful selection or source
helper generation cannot authorize provider execution, stage1/stage2 equality,
compiler correctness, general reproducibility, or release readiness.
