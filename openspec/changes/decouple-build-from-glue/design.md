## Context

crunch-build depends on crunch-glue for `KnownPaths`. KnownPaths mixes
conversion-time concerns (ATerm dedup, HDM cache, cycle detection) with
build-time concerns (derivation lookup, CA output resolution). The build
engine can't compile without the glue layer.

## Goals / Non-Goals

**Goals:** Remove crunch-build → crunch-glue dependency. Make crunch-build
an independent build engine that accepts pre-computed derivations.

**Non-Goals:** Change the conversion algorithm. Change the Worker/Goal
scheduler. Merge the types back together.

## Decisions

### 1. Split KnownPaths into two types

**Choice:**
- `ConversionCache` in crunch-glue: ATerm hash dedup, HDM cache,
  cycle detection. Methods: `begin_conversion()`, `end_conversion()`,
  `get_hdm_by_drv_path()`, `insert()`.
- `DerivationRegistry` in crunch-build: drv path lookup, CA output
  resolution. Methods: `get()`, `insert()`, `resolve_output()`,
  `get_by_drv_path()`.

**Rationale:** The two use cases have different lifetimes (conversion
is a one-shot phase, the registry persists through the build) and
different callers (convert() vs Worker/Builder). Splitting makes
each type's invariants clearer.

### 2. Pipeline bridges the gap

**Choice:** A `populate_registry()` function (in crunch-pipeline or
as a method on DerivationRegistry) that reads ConversionCache entries
and inserts them into the registry.

**Rationale:** The conversion output is a set of (aterm_hash, drv_path,
hdm, derivation, content_addressed) tuples. The registry consumes the
same data minus the conversion-specific fields. The bridge is trivial.

### 3. DerivationRegistry in crunch-build, not crunch-store

**Choice:** The registry lives in crunch-build.

**Rationale:** It's a build-time data structure — the Worker and Builder
use it for dependency resolution and CA tracking. It doesn't persist
across process restarts (CA mappings persist separately in crunch-store).
The store crate doesn't need it.

**Alternative:** Put it in crunch-store so both build and pipeline can
use it. Rejected because the build crate is the primary consumer, and
the pipeline just populates it.

## Risks / Trade-offs

**[Migration size]** Every file that imports KnownPaths changes. Worker,
Builder, build_request, dynamic — all take DerivationRegistry instead.
Mitigated by mechanical renaming.

**[Duplicate fields]** Some data exists in both ConversionCache and
DerivationRegistry (drv_path, derivation). Accepted — the duplication
is small, and the separation of concerns is worth it.
