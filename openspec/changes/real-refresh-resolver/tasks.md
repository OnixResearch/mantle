## Phase 1: LiveResolver Core

- [ ] Add `LiveResolver` struct to `src/project_cmd.rs` (or `src/resolve.rs` if >70 lines)
- [ ] Implement `resolve_git_rev` — `git ls-remote` subprocess, parse SHA, handle branch/tag/rev ref types
- [ ] Implement `hash_url_content` — reqwest blocking download with size limit, streaming hash, SRI encoding
- [ ] Handle peeled tags (`^{}` lines in ls-remote output)
- [ ] Return clear error when `git` not on PATH

## Phase 2: Wire Into CLI

- [ ] Replace `StubResolver` with `LiveResolver` in `cmd_refresh` and `cmd_list_stale`
- [ ] Delete `StubResolver` entirely
- [ ] Verify `cmd_refresh` writes updated lock + regenerates inputs.ncl after real resolution

## Phase 3: Tests

- [ ] Unit tests for git ls-remote output parsing (branch, tag, peeled tag, rev validation, empty output, malformed output)
- [ ] Unit tests for SRI encoding across sha256, sha512, blake3
- [ ] Integration test: create local bare git repo, commit, resolve branch ref, verify SHA matches
- [ ] Integration test: serve file via `file://` URL, refresh tarball input, verify hash in lockfile
- [ ] Integration test: unreachable input + reachable input, verify partial resolution + error report
- [ ] Integration test: `crunch list-stale` detects upstream change without modifying files
