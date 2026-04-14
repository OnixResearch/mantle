# Implementation plan: Reduce build pipeline overhead

This change is easiest to land as five small PRs with one behavioral move per
PR and focused regression coverage.

## PR 1: Direct derivation extraction helper

**Goal:** add a direct typed extraction API without changing the active build
path yet.

**Scope**
- add a derivation-focused evaluation helper in `crates/crunch-eval/src/lib.rs`
  so the direct typed representation is exposed by `crunch-eval`
- support both top-level shapes:
  - single derivation record
  - record of named derivations
- prove nested derivation inputs with Nickel enum tags deserialize correctly

**Files**
- `crates/crunch-eval/src/lib.rs`
- `crates/crunch-glue/src/types.rs` if helper support is needed
- tests under `crates/crunch-eval/` or `crates/crunch-pipeline/`

**Why first**
- isolates the highest-ROI hot path
- gives parity tests before switching the pipeline

**Acceptance**
- `crunch-eval` exposes the direct helper
- current pipeline still passes unchanged
- new tests cover single root, package set, nested enum-tag inputs

## PR 2: Switch build path off JSON round-trip

**Goal:** make `crunch-pipeline::build()` use the direct helper from PR 1.

**Scope**
- replace `evaluate_to_json()` + JSON deserialization in
  `crates/crunch-pipeline/src/lib.rs`
- keep `crunch eval` and other debug/reporting paths on explicit JSON export
- remove now-dead JSON-only build-path helpers if safe

**Files**
- `crates/crunch-pipeline/src/lib.rs`
- `src/main.rs` only if shared helper plumbing changes
- focused pipeline tests

**Acceptance**
- build path no longer exports full JSON just to obtain derivations
- `crunch eval` output stays unchanged
- pipeline tests pass for single root, package set, and partial failure cases

## PR 3: Session source-resolution cache

**Goal:** stop repeating closure walks and disk ingestion within one build
session.

**Scope**
- memoize source closure expansion in `resolve_and_ingest_sources()`
- replace `Vec.contains()`-style dedup with set-based tracking
- reuse known `PathInfo.node` / `output_nodes` data before re-ingesting the
  same path from disk

**Files**
- `crates/crunch-build/src/orchestrate.rs`
- maybe `crates/crunch-store/src/handle.rs` for helper accessors
- build-engine regression tests

**Acceptance**
- repeated source inputs in one session resolve once
- repeated on-disk dependency outputs are not re-ingested when castore content
  is already known
- tests prove reuse behavior directly

## PR 4: Metadata-only remote closure lookup

**Goal:** let closure walking read remote `References:` without downloading
full NAR content.

**Scope**
- add metadata-only narinfo lookup in store/remote-cache layer
- route `resolve_closure()` through metadata lookup for remote fallback
- keep actual substitution path unchanged for real cache hits

**Files**
- `crates/crunch-store/src/closure.rs`
- `crates/crunch-store/src/handle.rs` if helper lives there
- vendored `vendor/snix-store/src/pathinfoservice/nix_http.rs` or a wrapper
  around it
- store tests for remote closure walking

**Acceptance**
- closure walk can use remote narinfo metadata without NAR ingestion
- substitution still verifies and ingests full NAR payload on actual cache hit
- tests cover metadata hit, metadata miss, and graceful fallback

## PR 5: Worker hot-path ownership cleanup

**Goal:** reduce clone churn in ready, dispatch, and completion paths.

**Scope**
- move `BuildRequest` into spawned build task instead of cloning it
- drain waiter vectors with `mem::take` instead of cloning them on complete/
  fail propagation
- if still worthwhile after that, switch ready-path derivation ownership to
  shared ownership (`Arc<Derivation>` or equivalent)

**Files**
- `crates/crunch-build/src/worker.rs`
- `crates/crunch-build/src/goal.rs`
- `crates/crunch-build/src/orchestrate.rs` if ownership types change
- worker tests

**Acceptance**
- no `BuildRequest` clone on dispatch path
- waiter propagation stops cloning full waiter vectors
- worker tests still pass, including dynamic-derivation cases

## Suggested merge order

1. PR 1 helper only
2. PR 2 pipeline switch
3. PR 3 source/session cache
4. PR 4 metadata-only closure lookup
5. PR 5 worker ownership cleanup

## Suggested measurement points

Before PR 1 and after each PR, capture at least:
- wall time for a small package-set build
- wall time for `crunch self-build` stage0 on same host if feasible
- counters or spans for:
  - eval/extract time
  - source closure walks
  - disk ingests of repeated paths
  - remote closure metadata lookups
  - worker dispatch/finalization counts
