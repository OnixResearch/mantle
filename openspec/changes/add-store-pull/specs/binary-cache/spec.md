## ADDED Requirements

### Requirement: Store pull from directory

The store layer MUST provide a library function that imports PathInfo entries
from a flat Nix binary cache directory layout into the local store.

For each `.narinfo` file in the source directory the function MUST:
1. Parse the narinfo with the local store directory prefix
2. Verify at least one signature against trusted public keys (unless
   `trust_unsigned` is set)
3. Reject the path if the narinfo `StorePath` prefix does not match the
   local store directory
4. Skip the path if a PathInfo already exists locally for that store path
5. Resolve the NAR file path from the narinfo `URL` field
6. Ingest the NAR into castore via `ingest_nar_and_hash`, verifying the
   NAR hash matches the narinfo `NarHash`
7. Construct and persist a PathInfo from the narinfo fields
8. Export the castore node to the local store directory on disk

The function MUST return a structured `PullReport` with counts of imported
paths, skipped-already-present, skipped-untrusted, skipped-hash-mismatch,
skipped-missing-nar, skipped-parse-error, and total bytes ingested.

#### Scenario: Pull a single signed path from a pushed cache

- GIVEN a cache directory produced by `crunch store push` with one signed path
- AND the local store has no PathInfo for that path
- AND the narinfo signature matches a trusted public key
- WHEN `import_paths_from_cache_dir(handle, source, options)` is called
- THEN the NAR is ingested into castore
- AND a PathInfo is persisted in the local store
- AND the output is exported to the local store directory on disk
- AND the report shows `imported_count = 1`

#### Scenario: Pull skips already-present paths

- GIVEN a cache directory with one narinfo for path `hello`
- AND the local store already has PathInfo for `hello`
- WHEN `import_paths_from_cache_dir` is called
- THEN no NAR is ingested for `hello`
- AND the report shows `skipped_already_present_count = 1`

#### Scenario: Pull rejects untrusted signatures by default

- GIVEN a cache directory with one narinfo signed by key `unknown-1`
- AND the trusted public keys list does not include `unknown-1`
- AND `trust_unsigned` is false
- WHEN `import_paths_from_cache_dir` is called
- THEN no NAR is ingested
- AND the report shows `skipped_untrusted_count = 1`

#### Scenario: Pull accepts untrusted when trust_unsigned is set

- GIVEN a cache directory with one unsigned narinfo
- AND `trust_unsigned` is true
- WHEN `import_paths_from_cache_dir` is called
- THEN the NAR is ingested and PathInfo is persisted
- AND the report shows `imported_count = 1`

#### Scenario: Pull detects NAR hash mismatch

- GIVEN a cache directory where the NAR file has been corrupted
- AND the narinfo `NarHash` no longer matches the NAR content
- WHEN `import_paths_from_cache_dir` is called
- THEN the path is skipped
- AND the report shows `skipped_hash_mismatch_count = 1`

#### Scenario: Pull skips missing NAR files

- GIVEN a cache directory where the `.narinfo` exists but the referenced
  NAR file is absent
- WHEN `import_paths_from_cache_dir` is called
- THEN the path is skipped
- AND the report shows `skipped_missing_nar_count = 1`

#### Scenario: Pull rejects store prefix mismatch

- GIVEN a cache directory with narinfos using `StorePath: /nix/store/...`
- AND the local store uses prefix `/crunch/store`
- WHEN `import_paths_from_cache_dir` is called
- THEN those paths are skipped
- AND the report includes a store prefix mismatch indication

#### Scenario: Pull multiple paths

- GIVEN a cache directory with three valid signed narinfos
- WHEN `import_paths_from_cache_dir` is called with no path filter
- THEN all three paths are ingested and persisted
- AND the report shows `imported_count = 3`

#### Scenario: Pull with path filter

- GIVEN a cache directory with narinfos for paths `a`, `b`, and `c`
- AND the caller requests only `[a, c]`
- WHEN `import_paths_from_cache_dir` is called
- THEN only paths `a` and `c` are imported
- AND path `b` is not ingested

### Requirement: CLI store pull subcommand

The CLI MUST provide `crunch store pull` for importing paths from a binary
cache directory into the local store.

Required arguments:
- `--from <path>` — source cache directory (required)
- `<path>...` — store paths to import (optional if `--all` is given)
- `--all` — import all paths from the cache directory
- `--trusted-public-keys <key>...` — trusted public keys for signature
  verification (name:base64 format)
- `--trust-unsigned` — accept unsigned narinfos

The command MUST use the store mutation lock to prevent concurrent pull and
GC operations.

#### Scenario: Pull named paths from directory

- GIVEN a cache directory with pushed paths `hello` and `world`
- WHEN `crunch store pull --from /srv/cache hello world` runs
- THEN both paths are imported into the local store
- AND the command prints a summary of imported paths and bytes

#### Scenario: Pull all paths

- GIVEN a cache directory with five narinfos
- WHEN `crunch store pull --all --from /srv/cache` runs
- THEN all five paths are imported

#### Scenario: Pull with no trusted keys warns

- GIVEN a cache directory with signed narinfos
- AND no `--trusted-public-keys` and no `--trust-unsigned`
- WHEN `crunch store pull --all --from /srv/cache` runs
- THEN the command uses the store's configured trusted keys if any
- AND paths whose signatures do not match are skipped with a warning

#### Scenario: Pull from nonexistent directory

- GIVEN `--from /nonexistent/dir`
- WHEN `crunch store pull --from /nonexistent/dir --all` runs
- THEN the command fails with a clear error message
