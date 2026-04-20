## ADDED Requirements

### Requirement: Store pull from HTTP cache

The store layer MUST provide a library function that imports PathInfo entries
from a remote HTTP/HTTPS Nix binary cache into the local store.

For each requested store path the function MUST:
1. Construct the narinfo URL as `<cache-url>/<nixbase32(digest)>.narinfo`
2. Fetch and parse the narinfo, verifying at least one signature against
   trusted public keys (unless `trust_unsigned` is set)
3. Skip the path if a PathInfo already exists locally for that store path
4. Resolve the NAR URL from the narinfo `URL` field relative to the cache URL.
   Absolute URLs in the `URL` field MUST be rejected to prevent SSRF; only
   relative paths (e.g., `nar/<hash>.nar.xz`) are resolved
5. Download the NAR, decompressing if the narinfo `Compression` field
   specifies bzip2, gzip, xz, or zstd
6. Ingest the NAR into castore via `ingest_nar_and_hash`, verifying the
   NAR hash matches the narinfo `NarHash`
7. Construct and persist a PathInfo from the narinfo fields
8. Export the castore node to the local store directory on disk

The function MUST return the same structured `PullReport` type used by
directory pull. HTTP-specific skip reasons MUST map to existing report fields:
narinfo 404 → `skipped_missing_nar_count`, narinfo parse failure →
`skipped_parse_error_count`, network error → `skipped_parse_error_count`.

Phase 1 imports only the explicitly requested paths. Missing references
(narinfo `References` entries not in the local store) are NOT fetched
recursively.

#### Scenario: Pull a single path from HTTP cache

- GIVEN a remote HTTP cache at `http://cache.example.com` with one signed path
- AND the local store has no PathInfo for that path
- AND the narinfo signature matches a trusted public key
- WHEN `import_paths_from_http_cache(handle, url, [path], options)` is called
- THEN the narinfo is fetched from `http://cache.example.com/<digest>.narinfo`
- AND the NAR is downloaded and ingested into castore
- AND a PathInfo is persisted in the local store
- AND the report shows `imported_count = 1`

#### Scenario: Pull skips 404 paths

- GIVEN a remote HTTP cache that returns 404 for `<digest>.narinfo`
- WHEN `import_paths_from_http_cache` is called for that path
- THEN the path is skipped
- AND the report shows `skipped_missing_nar_count = 1`

#### Scenario: Pull handles compressed NAR

- GIVEN a remote HTTP cache serving xz-compressed NARs
- AND the narinfo `Compression` field is `xz`
- WHEN `import_paths_from_http_cache` is called
- THEN the NAR is decompressed during ingestion
- AND the decompressed NAR hash matches the narinfo `NarHash`

#### Scenario: Pull rejects untrusted signatures over HTTP

- GIVEN a remote cache with narinfos signed by an unknown key
- AND `trust_unsigned` is false
- WHEN `import_paths_from_http_cache` is called
- THEN no NAR is downloaded
- AND the report shows `skipped_untrusted_count = 1`

#### Scenario: Network error during HTTP pull

- GIVEN a remote cache that is unreachable
- WHEN `import_paths_from_http_cache` is called for a path
- THEN the path is skipped with a warning
- AND the pull continues for remaining paths

### Requirement: Remote nix-cache-info validation

Before importing any paths over HTTP, the pull function MUST fetch
`<cache-url>/nix-cache-info` and validate that the remote `StoreDir` matches
the local store directory prefix.

If the remote `StoreDir` does not match, the function MUST return an error
without fetching any narinfos.

If `nix-cache-info` is absent or unparseable, the function MUST proceed with
a warning (some caches omit this file).

#### Scenario: StoreDir matches

- GIVEN a remote cache with `nix-cache-info` containing `StoreDir: /nix/store`
- AND the local store uses prefix `/nix/store`
- WHEN HTTP pull begins
- THEN narinfo fetching proceeds normally

#### Scenario: StoreDir mismatch is a hard error

- GIVEN a remote cache with `nix-cache-info` containing `StoreDir: /nix/store`
- AND the local store uses prefix `/crunch/store`
- WHEN HTTP pull begins
- THEN the function returns an error before any narinfo is fetched

#### Scenario: Missing nix-cache-info proceeds with warning

- GIVEN a remote cache that returns 404 for `/nix-cache-info`
- WHEN HTTP pull begins
- THEN a warning is logged
- AND narinfo fetching proceeds assuming the remote StoreDir matches the local
  prefix (narinfo-level parse failures will catch actual mismatches)

## MODIFIED Requirements

### Requirement: CLI store pull subcommand

The CLI `crunch store pull` `--from` argument MUST accept both local directory
paths and `http://` or `https://` URLs.

When `--from` is a URL:
- The HTTP pull path MUST be used
- `--all` MUST be rejected with a clear error ("--all is not supported for
  HTTP caches; specify paths explicitly")
- Path selectors are required

When `--from` is a local path:
- Behavior is unchanged from the directory pull implementation

#### Scenario: Pull from HTTP URL

- GIVEN `--from https://cache.example.com`
- AND store path arguments are provided
- WHEN `crunch store pull` runs
- THEN paths are fetched over HTTP

#### Scenario: Pull --all from HTTP is rejected

- GIVEN `--from https://cache.example.com`
- AND `--all` is passed
- WHEN `crunch store pull` runs
- THEN the command fails with an error message explaining that --all is not
  supported for HTTP caches
