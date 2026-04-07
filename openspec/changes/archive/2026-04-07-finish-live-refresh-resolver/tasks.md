## Phase 1: Resolver and API core

These tasks implement the existing `project-management` requirements and the
CLI delta in this change.


- [x] Replace the stub refresh path with a `LiveResolver` owned by the binary crate
- [x] Extend the project-management stale API so it preserves failed checks separately from stale inputs
- [x] Implement git ref resolution for branch, tag, and rev inputs via `git ls-remote`
- [x] Implement hash resolution with the correct semantics for files, tarballs, git checkouts, and local patch files

## Phase 2: CLI wiring

- [x] Wire `crunch refresh` to the live resolver and keep partial-success updates visible in output
- [x] Make `crunch refresh` exit non-zero when any selected input fails to resolve or hash
- [x] Wire `crunch list-stale` to the live resolver and print stale items and failed checks distinctly
- [x] Prevent `crunch list-stale` from printing `all inputs up to date` when any check failed

## Phase 3: Verification

- [x] Add unit coverage for git `ls-remote` parsing and stale/failure aggregation
- [x] Add integration coverage for local git refresh, `file://` tarball refresh, local patch hashing, and mixed success/failure refresh runs
- [x] Verify that refreshed lock entries match the hash semantics accepted by the existing fetch/build helpers
