## Context

`crunch-project` keeps refresh logic pure by delegating I/O to a
`RefreshResolver`. The current shape is already larger than the original
proposal assumed:

```rust
pub trait RefreshResolver {
    fn resolve_git_rev(
        &self,
        repository: &str,
        reference: &GitReference,
    ) -> Result<Option<String>, Error>;

    fn hash_url_content(
        &self,
        url: &str,
        algo: &HashAlgo,
    ) -> Result<Option<String>, Error>;

    fn hash_local_file(
        &self,
        path: &str,
        algo: &HashAlgo,
    ) -> Result<Option<String>, Error>;
}
```

Even with that third hook, the current abstraction is still incomplete:

- `resolve_input()` can fill file/tarball hashes from `hash_url_content()`
  and git revs from `resolve_git_rev()`, but git lock hashes still fall
  back to `input.hash.expected` or an empty string
- `list_stale()` collapses refresh results to `Vec<String>`, so stale
  checks lose failure details
- the binary crate still provides `StubResolver`, and local patch hashing
  falls back to the default `Ok(None)` implementation

This change therefore needs both a live resolver and a small refresh API
redesign.

## Goals / Non-Goals

**Goals:** make `crunch refresh` and `crunch list-stale` work for real,
match existing fetch semantics for file/tarball/git inputs, lock local
patches, and stop misreporting failed stale checks as "up to date".

**Non-Goals:** async resolution, parallel input resolution, caching beyond
the lockfile, retry logic, auth/token plumbing, progress reporting.

## Decisions

### 1. Expand the refresh/stale abstraction instead of forcing the old signatures

**Choice:** Update `crunch-project`'s refresh/stale API as needed so the
pure core can request the exact facts it needs and can return structured
results.

That means the final API must be able to represent all of these cases:
- git reference resolution
- git checkout hashing
- file byte hashing
- tarball tree hashing
- local patch hashing
- stale-check failures alongside stale inputs

Whether that ends up as more resolver methods or a richer per-input
resolution API is an implementation detail. The important part is that the
interface distinguishes plain files from tree-producing sources and does
not erase failures.

**Rationale:** The current API cannot express a valid refreshed git lock
entry, and `Vec<String>` stale results cannot distinguish "nothing stale"
from "everything failed".

**Alternative:** Keep the existing API and stuff git hashes/stale errors
through ad hoc side channels. Rejected — that would preserve the exact bugs
this change is meant to fix.

### 2. Keep the imperative shell in the binary crate

**Choice:** The concrete live resolver stays in the binary crate,
implemented in `src/project_cmd.rs` or extracted to `src/resolve.rs` if it
grows past the style limits.

**Rationale:** subprocesses, tempdirs, downloads, and path-based hashing are
all shell concerns. `crunch-project` remains the functional core.

**Alternative:** Move resolution into `crunch-project`, `crunch-build`, or
`crunch-pipeline`. Rejected — that would either add I/O to the pure crate
or invert existing dependencies.

### 3. Git refresh is two-step: resolve the selector, then hash the checkout

**Choice:** For git inputs, refresh does two distinct things:
1. Resolve the reference to a concrete commit SHA.
2. Materialize that commit into a temp checkout and compute the
   recursive/NAR hash that `crunch.fetchGit` expects.

Branch and tag refs use `git ls-remote`. Rev refs skip the network lookup
but still validate that the rev is a full 40-character hex SHA.

**Rationale:** Recording only the rev is not enough. The lockfile also needs
an actual hash value, and downstream fetch/build code expects that hash to
cover the checkout tree, not just the rev string.

**Alternative:** Only run `git ls-remote` and leave the hash untouched or
empty. Rejected — that still produces invalid or stale git lock entries.

### 4. Hash semantics must match the existing fetch helpers exactly

**Choice:** Refresh uses different hashing paths for different source kinds:
- file inputs -> flat hash of downloaded bytes
- tarball inputs -> unpack, then recursive/NAR hash of the unpacked tree
- git inputs -> checkout, then recursive/NAR hash of the working tree
- local patches -> flat hash of the patch file on disk
- remote patches -> flat hash of downloaded patch bytes

The implementation should reuse or factor shared helper logic from the
existing fetch/build code where practical so refresh and build do not drift.

**Rationale:** `crunch.fetchTarball` and `crunch.fetchGit` already define the
source-of-truth semantics. Refresh must lock values that those helpers will
accept later.

**Alternative:** Treat every remote URL as "download bytes and hash them".
Rejected — that is wrong for tarballs and git trees.

### 5. Local patch locking is part of "live" refresh

**Choice:** The live resolver must implement local file hashing and treat
manifest local patch paths as real inputs to refresh.

The resolver should resolve local patch paths relative to the project root,
not the current working directory of a random subprocess.

**Rationale:** `apply_outcomes()` already resolves manifest patches into
`Lockfile.patches`. A live resolver that leaves `hash_local_file()` as the
default no-op still produces incomplete lockfiles.

**Alternative:** Leave local patch locking to a future change. Rejected —
that would keep refresh half-functional.

### 6. Stale reporting must preserve failures

**Choice:** Replace the current "just names" stale result with a structured
report that can carry both stale inputs and failed checks.

The CLI should print stale items and failures distinctly. If any check
fails, it must not print `all inputs up to date`, and it should exit
non-zero after reporting what it did learn.

**Rationale:** An empty stale list is ambiguous today. Structured reporting
makes partial success visible instead of silently optimistic.

**Alternative:** Keep `Vec<String>` and rely on stderr side effects. Rejected
— the API itself would still erase the information the CLI needs.

### 7. Error handling stays per-input, not abort-all

**Choice:** Refresh and stale detection continue to resolve as many inputs as
possible. One failed URL or repo must not block unrelated inputs.

**Rationale:** This matches current `RefreshOutcome::Failed` behavior and is
friendlier for real projects with many inputs.

## Risks / Trade-offs

**[API churn in `crunch-project`]** -> Acceptable. The current API is not
strong enough for the required behavior.

**[Git not on PATH]** -> Return a clear error naming the missing binary.
This is already a runtime requirement for git-backed fetches.

**[Temp space / large sources]** -> Tarball and git hashing may need temp
materialization. Reuse the existing size limits and fail clearly when
limits are exceeded.

**[Logic drift between refresh and build]** -> Prefer shared helpers or
factored common code over copy/pasted hashing rules.

**[Blocking I/O in sync context]** -> Acceptable. The trait is synchronous
and these commands are CLI-driven, not latency-sensitive RPCs.
