## Why

`crunch refresh` is wired end-to-end — manifest parsing, lock merging,
stale detection, generated inputs, CLI commands — but the `RefreshResolver`
in `project_cmd.rs` is a stub. Every call to `crunch refresh` succeeds
without resolving anything real.

The review turned up a second problem: the current refresh abstraction is
too weak for the lockfile semantics the build path already expects.
Today:

- git refresh can discover a rev, but it still has no way to compute the
  recursive checkout hash required by `crunch.fetchGit`
- tarball refresh is easy to specify incorrectly as a flat hash of the
  downloaded archive, even though `crunch.fetchTarball` locks a recursive
  hash of the unpacked tree
- local patch locking depends on `RefreshResolver::hash_local_file()`, so
  a resolver that only handles git and remote URLs still leaves patch data
  unresolved
- `list-stale` collapses everything to a bare list of names, so failures
  are easy to misreport as "all inputs up to date"

This change needs to fix the real workflow, not just swap in a non-stub
implementation.

## What Changes

- Replace `StubResolver` in `src/project_cmd.rs` with a `LiveResolver`
  that performs real I/O.
- Extend the `crunch-project` refresh/stale abstraction as needed so the
  pure core can request the facts it actually needs: git ref resolution,
  git checkout hashing, file hashing, tarball tree hashing, local patch
  hashing, and structured stale/failure reporting.
- For git inputs, resolve branch/tag refs with `git ls-remote`, validate
  pinned rev refs, and compute the recursive/NAR hash of the checkout at
  the resolved rev.
- For file vs tarball inputs, match existing fetch semantics exactly:
  file inputs hash raw bytes; tarball inputs hash the unpacked tree.
- Hash local patches during refresh so `apply_outcomes()` can write valid
  `Lockfile.patches` entries.
- Make `crunch list-stale` report stale inputs and failed checks
  separately, instead of treating every failure as "up to date".
- Add integration tests for local git repos, tarball hashing, patch
  locking, partial failures, and stale reporting.

## Capabilities

### Modified Capabilities
- `project-management`: refresh resolves real upstream state and writes
  lock data that matches the fetch/build pipeline
- `cli`: `crunch refresh` and `crunch list-stale` produce meaningful
  output, including partial-failure reporting

## Impact

- **Files**: `src/project_cmd.rs` or a new `src/resolve.rs`,
  `crates/crunch-project/src/refresh.rs`, and CLI/integration tests in
  `tests/project_cli.rs` or a new `tests/refresh_integration.rs`
- **APIs**: `crunch-project`'s refresh/stale reporting API will likely
  change. The current resolver and `list_stale()` signatures do not carry
  enough information for git hashes or stale-check failures.
- **Dependencies**: `git` on PATH remains required for git inputs. Rust
  dependencies may be reused from existing fetch helpers or factored into
  shared helpers.
- **Testing**: local bare git repo, local tarball via `file://`, temp patch
  files, and mixed reachable/unreachable inputs to avoid network flakiness

## Out of Scope

- Async/concurrent refresh
- Caching resolved values across runs beyond the lockfile
- New input kinds
- Auth/token config for private repos
- Progress bars or streaming output during resolution
