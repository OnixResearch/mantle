# ADR 0084: Version dynamic plans for output-owned source slices

## Status

Proposed (2026-09-30). This isolated candidate reconstructs the v2 ABI and
signed-batch worker/store path on pinned main. A private dev46 two-run CLI
fixture admitted both v2 plans and kept one slice path and unit derivation path
stable while the producer output changed. Candidate-wide strict gates remain
red on unrelated pinned-main findings; this decision is not Accepted.

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

The pure ABI can be exercised independently while v1 remains byte-for-byte unchanged. Extra NAR hashing and store objects are bounded by count, path length/depth, and aggregate NAR bytes. Worker and store regression fixtures cover signed publication, deduplication, producer reruns, rejection without partial registration, and conflicting batch candidates. These fixtures are not substitute evidence for an executed isolated-candidate gate. Snix Redb's logical PathInfo batch does **not** guarantee physical castore cleanup or cross-process no-clobber: another signer may overwrite a preflighted digest before its unconditional insert. Even successful slice admission proves only observed subtree identity and publication, not producer correctness or source trust.
