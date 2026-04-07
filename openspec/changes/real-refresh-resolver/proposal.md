## Why

`crunch refresh` is wired end-to-end — manifest parsing, lock merging,
stale detection, generated inputs, CLI commands — but the `RefreshResolver`
in `project_cmd.rs` is a stub that returns `Ok(None)` for both
`resolve_git_rev` and `hash_url_content`. Every call to `crunch refresh`
succeeds without changing anything. The project-management workflow is
structurally complete but functionally inert.

The resolver trait was designed to be implemented by the caller so
crunch-project stays pure (no I/O, no async, no network). The binary
crate is the right place for the concrete implementation. The pieces it
needs already exist elsewhere in the workspace:

- `crunch-build::fetcher` downloads URLs, computes content hashes, and
  extracts tarballs.
- `git ls-remote` resolution is a single subprocess call.
- The existing `FetchBuildService` ingests downloaded content into
  castore and computes NAR/flat hashes — exactly what
  `hash_url_content` needs.

The work is wiring, not invention.

## What Changes

- Replace `StubResolver` in `src/project_cmd.rs` with a `LiveResolver`
  that performs real network I/O.
- `resolve_git_rev`: run `git ls-remote <repo> <ref>` and parse the
  output. Return the full commit SHA for branch/tag refs, validate
  format for rev refs.
- `hash_url_content`: download the URL to a temp file, hash it with the
  requested algorithm, return the SRI string. Reuse the existing
  download helpers from `crunch-build::fetcher` (or equivalent logic —
  reqwest + hash streaming).
- Add integration tests that exercise refresh with a local git repo
  and a local HTTP/file server.
- Remove `StubResolver` entirely — no fallback to no-op behavior.

## Capabilities

### Modified Capabilities
- `project-management`: refresh now resolves inputs for real instead of
  returning no-ops
- `cli`: `crunch refresh` and `crunch list-stale` produce meaningful
  output

## Impact

- **Files**: `src/project_cmd.rs` (replace StubResolver with LiveResolver),
  possibly a new `src/resolve.rs` if the resolver grows beyond ~70 lines.
  Integration tests in `tests/project_cli.rs` or a new
  `tests/refresh_integration.rs`.
- **APIs**: no public API changes in crunch-project (the trait is already
  defined). Only the binary-crate implementation changes.
- **Dependencies**: `reqwest` already in the workspace for fetcher.
  `git` binary required on PATH (already needed for fetchGit). No new
  deps expected.
- **Testing**: local git repo (init + commit + push to bare) for
  `resolve_git_rev`. Local `file://` URL or tiny HTTP server for
  `hash_url_content`. Both avoid network flakiness.

## Out of Scope

- Async/concurrent refresh (resolving multiple inputs in parallel)
- Caching resolved values across runs (the lock IS the cache)
- New input kinds (channel, registry, etc.)
- Signing or verifying fetched content
- Progress bars or streaming output during resolution
