# Design: Reduce build pipeline overhead

## Context

The current build path pays avoidable coordination cost in four places:

1. `crunch-pipeline::build()` still uses `crunch_eval::evaluate_to_json()` and
   `serde_json` even though the derivation schema now has direct Nickel
   deserialization support.
2. `resolve_and_ingest_sources()` repeats closure walks and path ingestion for
   source inputs that appear in multiple derivations within the same build.
3. remote closure fallback currently goes through `PathInfoService::get()`, and
   the HTTP implementation downloads and ingests a full NAR before returning
   references.
4. the worker still clones `Derivation`, `BuildRequest`, and waiter lists on
   the ready -> building -> done path.

These costs stack up in self-build and package-set builds where the pipeline
handles many derivations before the expensive sandbox work even starts.

## Goals / Non-Goals

**Goals:**

- remove whole-program JSON export from the derivation build path
- reuse source-closure and input-node work within a build session
- separate remote reference discovery from full substitution downloads
- reduce scheduler hot-path allocation and clone churn

**Non-Goals:**

- redesign build output hashing or CA rewrite algorithms yet
- add dependency-layer caching for cargo or self-build outputs
- change CLI output shape or FOD mismatch behavior
- parallelize `prepare_build()` and `finish_build()` in this change

## Decisions

### 1. Use direct typed derivation extraction in the pipeline

**Choice:** `crunch-pipeline::build()` will consume evaluated Nickel values
through a direct typed extraction helper instead of `evaluate_to_json()` when
it is extracting `CrunchDerivation` roots for build execution.

**Rationale:** the derivation schema already has the `NickelString` support
needed for direct `Expr::to_serde()`-style deserialization. Keeping JSON export
on the build path only adds serialization, allocation, and parsing work.

**Implementation:** add a derivation-focused helper in `crunch-eval` or
`crunch-pipeline` that handles both single derivations and package sets while
leaving `crunch eval` JSON output unchanged.

### 2. Memoize source closure expansion and reusable nodes per build session

**Choice:** source closure resolution and reusable input-node materialization
will be cached for the lifetime of one build session.

**Rationale:** the closure of a given source path is pure within a single
session, and the same source paths commonly appear in many derivations.
Repeating the same closure walk and re-ingesting the same on-disk tree wastes
I/O without improving correctness.

**Implementation:** cache closure-member sets keyed by source path and cache
resolved input nodes keyed by store path when local `PathInfo` plus castore
content are already available.

### 3. Split remote closure metadata lookup from full substitution

**Choice:** closure walking will use a narinfo-metadata lookup path that reads
`References:` without downloading or ingesting the NAR payload. Full NAR
transfer stays on the substitution path only.

**Rationale:** closure discovery needs graph metadata, not content. Coupling
those concerns forces fresh-store builds to substitute data just to learn what
else to mount.

**Implementation:** extend the remote-cache layer with a metadata-only narinfo
query or equivalent store helper and route `resolve_closure()` through it.
Substitution verification and write-through behavior remain unchanged.

### 4. Reduce worker hot-path clones by moving or sharing data

**Choice:** worker dispatch will avoid deep cloning of derivations, build
requests, and waiter vectors where ownership can be shared or moved safely.

**Rationale:** scheduler hot paths should spend time scheduling, not duplicating
large maps and vectors. This cost scales badly in self-build-sized graphs.

**Implementation:** move `BuildRequest` into the spawned build task, drain
waiter vectors with `mem::take`, and use shared ownership for derivation data
where it simplifies ready/dispatch transitions.

## Risks / Trade-offs

**[Direct extraction drift]**
The direct typed path must preserve current single-derivation vs package-set
selection and must still accept nested derivation inputs that contain Nickel
enum tags.

**Mitigation:** add regression tests that compare direct extraction behavior to
current pipeline expectations for both top-level shapes and nested inputs.

**[Over-caching stale inputs]**
Session caches keyed too loosely could reuse the wrong closure or node.

**Mitigation:** keep caches scoped to one process-level build session and key
by logical store path identity, not by file-system path strings alone.

**[Metadata path weakening substitution trust]**
A narinfo-only lookup must not become an unchecked substitute path.

**Mitigation:** keep metadata-only closure discovery separate from the existing
substitution path that downloads content, verifies NAR metadata, and persists
PathInfo.
