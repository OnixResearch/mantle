## Phase 1: Refresh API and LiveResolver Core

- [ ] Extend `crunch-project`'s refresh/stale API so it can express git checkout hashing, tarball tree hashing, local patch hashing, and structured stale/failure reporting
- [ ] Add `LiveResolver` to `src/project_cmd.rs` or `src/resolve.rs`
- [ ] Implement git ref resolution via `git ls-remote`, including branch/tag/rev handling and peeled tags
- [ ] Implement git checkout hashing that matches `crunch.fetchGit` recursive hash semantics
- [ ] Implement remote file hashing with flat hash semantics
- [ ] Implement tarball download + unpack + recursive/NAR tree hashing that matches `crunch.fetchTarball`
- [ ] Implement `hash_local_file` for local patches, resolving manifest paths relative to the project root
- [ ] Return clear errors when `git` is not on PATH, a local patch is missing, or a stale check cannot be completed

## Phase 2: Wire Into CLI

- [ ] Replace `StubResolver` with `LiveResolver` in `cmd_refresh` and `cmd_list_stale`
- [ ] Delete `StubResolver` entirely
- [ ] Update `cmd_list_stale` to print stale inputs and failed checks distinctly and exit non-zero on failed checks
- [ ] Verify `cmd_refresh` writes updated lock data and regenerates `.crunch/inputs.ncl`
- [ ] Verify refreshed git and tarball entries carry hashes that the existing fetch/build path accepts
- [ ] Verify refresh resolves referenced local patches into `Lockfile.patches`

## Phase 3: Tests

- [ ] Unit tests for git `ls-remote` parsing (branch, tag, peeled tag, rev validation, empty output, malformed output)
- [ ] Unit tests for source-kind hash semantics (file flat hash, tarball tree hash, git tree hash)
- [ ] Unit tests for stale-report aggregation (stale + failed + clean cases)
- [ ] Integration test: create local bare git repo, refresh a branch ref, verify both rev and git tree hash in the lockfile
- [ ] Integration test: refresh a `file://` tarball input and verify the lockfile stores the unpacked-tree hash, not the archive-byte hash
- [ ] Integration test: local patch file is resolved relative to the project root, hashed, and locked during refresh
- [ ] Integration test: unreachable input + reachable input yields partial refresh plus a reported error
- [ ] Integration test: `crunch list-stale` reports stale inputs and failed checks without modifying project files
