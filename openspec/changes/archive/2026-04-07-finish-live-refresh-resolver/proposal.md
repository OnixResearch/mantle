## Why

`crunch refresh` and `crunch list-stale` still route through `StubResolver`
in `src/project_cmd.rs`. The current command surface exists, but the live
resolution path does not:

- git refs are not resolved with `git ls-remote`
- git and tarball inputs are not hashed with the same recursive semantics the
  fetch/build path expects
- local patch hashes are not populated during refresh
- stale-check failures can collapse into a misleading `all inputs up to date`
  result

The main specs already describe the intended behavior. The gap is in the
implementation and operator-facing failure handling.

## What Changes

- Replace `StubResolver` with a real `LiveResolver` in the binary crate
- Keep `crunch-project` pure while extending its refresh/stale API where the
  current return types are too weak
- Resolve git branch/tag refs with `git ls-remote`, validate pinned revs, and
  compute the recursive hash that `crunch.fetchGit` expects
- Distinguish flat file hashing from tarball/git tree hashing so lock entries
  match the build pipeline
- Hash local patch files relative to the project root during refresh
- Make `crunch refresh` and `crunch list-stale` report partial failures
  clearly and exit non-zero when checks fail
- Add integration coverage for local git, local tarballs, patch locking, and
  mixed success/failure refresh runs

## Capabilities

### Modified Capabilities
- `cli`: refresh and stale commands surface stale inputs and failed checks as
  different outcomes

## Impact

- **Files**: `src/project_cmd.rs` or a new resolver module, `crates/crunch-project/src/refresh.rs`, and CLI/integration tests under `tests/`
- **APIs**: `RefreshResolver` callers and stale-reporting types in
  `crunch-project`
- **Dependencies**: `git` on PATH for git inputs; existing download/temp-file
  helpers reused where possible
- **Testing**: local bare git repo, `file://` tarball, local patch files,
  and mixed reachable/unreachable inputs

## Notes

This is a follow-through change. The resolver, hashing, and stale-detection
semantics are already specified in `openspec/specs/project-management/spec.md`.
This change exists to bring the implementation into conformance with that main
spec and to add an explicit CLI delta for failure reporting under
`openspec/changes/finish-live-refresh-resolver/specs/cli/spec.md`.
