# Remove host git from fetchGit

## Sequence

Step 4 of 6. This change hardens the fetch path after hermeticity reporting is
in place.

## Why

`fetchGit` still depends on host `git` discovery through common paths and
`PATH`. That makes fetch semantics depend on ambient host tools and host `git`
configuration.

That is one of the largest remaining hermeticity leaks in crunch's external
source path.

## What Changes

- remove arbitrary host `git` discovery from `fetchGit`
- replace it with a crunch-controlled git materialization path
- make fetch errors depend on crunch's own fetch semantics, not host `git` stderr
- add regression tests proving that host `PATH` and host `git` version no longer affect `fetchGit`

## Capabilities

### New Capabilities

- `host-independent-fetchgit`: `fetchGit` no longer depends on arbitrary host `git` discovery

## Impact

- **Files**: `crates/crunch-build/src/fetcher.rs` and fetcher integration tests
- **Behavior**: `fetchGit` becomes controlled by crunch-owned semantics instead of host tool lookup
- **Testing**: add PATH- and host-tool-insensitivity coverage for `fetchGit`
