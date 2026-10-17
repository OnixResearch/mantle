# ADR 0084: Version dynamic plans for output-owned source slices

## Status

Proposed (2026-09-30). The pure v2 ABI, signed-batch worker/store path, scoped tests, and a real outside-only two-run CLI/sandbox identity fixture are implemented. Final root strict gates and isolated-branch archival remain pending; this decision is not Accepted.

## Context

`mantle-plan-v1` admits only existing store paths as source inputs. When a plan producer places several package roots inside one output, using that whole output as every unit's input couples otherwise independent derivation identities. ADR 0011 requires dynamic growth to be admitted by the worker after the producer completes. The v1 canonical bytes and digests are an existing compatibility contract.

## Decision Drivers

- A source slice must be content-bound without adding store authority inside a build sandbox.
- Producer output ownership and source identity must be explicit and bounded.
- Published paths must be stable when unrelated producer output bytes change.
- No rejected plan may leave partially published slices or registered units.

## Decision

Introduce `mantle-plan-v2` alongside, not inside, the unchanged v1 decoder. A v2 source is either the existing store-path form or `{id, producer_output, subpath, store_name, nar_blake3}`. The producer output must belong to the producing derivation; it cannot be the plan artifact output. Subpaths are non-empty relative sequences with no `.` or `..`, empty components, backslashes, NULs, or symlink traversal. Expected NAR BLAKE3 is required. Canonical sources order by source id, and the v2 plan digest binds the expected digest. The logical path is determined by the observed NAR content and declared store name under the active logical prefix, never by producer identity, output name, or subpath.

The pure planner takes worker-observed tree facts, rejects all failures before effects, and returns an ordered candidate batch. `BuildStore` can observe but cannot publish subtrees; the Builder retains its private signing key and an owned, narrow `SliceAdmission` capability distinct from `BuildStore`. The worker resolves bounded producer-output subpaths without following links, verifies the NAR BLAKE3 against the plan, then publishes the deduplicated signed PathInfos in one atomic logical batch before registering any unit or goal. `SourceAdmission::preflight` followed by repeated `SourceAdmission::ingest` is **not** atomic and must not be used to implement this decision. An uncertain post-commit result terminates the worker rather than claiming clean rejection. No store protocol enters the sandbox.

## Consequences

The pure ABI can be tested independently while v1 remains byte-for-byte unchanged. Extra NAR hashing and store objects are bounded by count, path length/depth, and aggregate NAR bytes. Focused worker tests prove signed publication, identical-content deduplication, unchanged slice and unit paths across two different producer outputs, and unchanged registry/goal/scheduler/rejection-report state for invalid slices. Store tests prove two-backend batch atomicity for a conflicting candidate. Snix Redb's logical PathInfo batch does **not** guarantee physical castore cleanup or cross-process no-clobber: another signer may overwrite a preflighted digest before its unconditional insert. Even successful slice admission proves only observed subtree identity and publication, not source trust, license, producer correctness, build success, or release eligibility.
