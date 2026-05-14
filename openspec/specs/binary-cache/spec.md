# Binary Cache Substitution Specification

## Purpose

Defines how mantle fetches pre-built outputs from remote Nix binary
caches, avoiding local builds when cached results are available.
## Requirements
### Requirement: Remote Cache Fallback

The build pipeline MUST query a remote binary cache when a derivation's
output is not found in the local PathInfo database.

#### Scenario: Cache hit from remote

- GIVEN a derivation `hello` with output path digest `D`
- AND the local PathInfo database has no entry for `D`
- AND `https://cache.nixos.org/D.narinfo` returns a valid narinfo
- WHEN `check_cache` runs for `hello`
- THEN the NAR is downloaded, decompressed, and ingested into castore
- AND a PathInfo entry is written to the local redb database
- AND the build is skipped (output marked as cached)

#### Scenario: Remote miss falls through to build

- GIVEN a derivation `mypkg` with output path digest `D`
- AND the local PathInfo database has no entry for `D`
- AND the remote cache returns 404 for `D.narinfo`
- WHEN `check_cache` runs for `mypkg`
- THEN the derivation is built locally as normal

#### Scenario: Subsequent build uses local cache

- GIVEN `hello` was previously substituted from a remote cache
- AND its PathInfo exists in the local redb database
- WHEN `check_cache` runs for `hello` again
- THEN the local PathInfo is used (no network request)

### Requirement: FOD Substitution Skip

The system MUST NOT attempt remote substitution for fixed-output
derivations (derivations where any output has a `ca_hash`).

#### Scenario: FOD bypasses remote cache

- GIVEN a fetchurl derivation with `fixed_output.hash = "sha256-..."`
- WHEN the build pipeline processes this derivation
- THEN no `.narinfo` request is made to the remote cache
- AND the derivation is built via the fetcher pipeline

### Requirement: Substitution Disable Flag

The CLI MUST provide a `--no-substitute` flag that disables all
remote cache lookups.

#### Scenario: No-substitute flag

- GIVEN `--no-substitute` is passed on the command line
- WHEN any derivation is processed
- THEN no remote cache requests are made
- AND all derivations are built locally

### Requirement: Substituter Configuration

The CLI MUST accept a `--substituters` flag specifying one or more
cache URLs.

#### Scenario: Custom substituter

- GIVEN `--substituters https://my-cache.example.com`
- WHEN a derivation output is not in local cache
- THEN `https://my-cache.example.com/<digest>.narinfo` is queried

#### Scenario: Default substituter

- GIVEN no `--substituters` flag is provided
- AND `--no-substitute` is not provided
- WHEN a derivation output is not in local cache
- THEN `https://cache.nixos.org/<digest>.narinfo` is queried

### Requirement: Remote Failure Graceful Degradation

Remote cache failures MUST be treated as cache misses, not build failures,
including network errors, malformed narinfo, and NAR download failures.

#### Scenario: Network error during substitution

- GIVEN the remote cache is unreachable
- WHEN `check_cache` queries the remote
- THEN a warning is logged
- AND the derivation is built locally

### Requirement: CA Derivation Substitution Skip

The system MUST NOT attempt remote substitution for content-addressed
derivations whose output paths are not yet known (output.path is None).

#### Scenario: CA derivation not substituted

- GIVEN a CA derivation `ca-hello` with `output.path = None`
- WHEN `check_cache` runs
- THEN no remote cache request is made for this output
- AND the derivation proceeds to build

### Requirement: Delta-aware Substitution Negotiation

The substitution pipeline MUST allow a trusted remote cache to advertise a
delta-capable transfer path in addition to ordinary full-artifact fetch.

A delta-capable HTTP cache MUST expose its delta negotiation and streaming
endpoints under the same cache authority used for ordinary substitution.

When both sides support delta transfer, mantle MUST prefer the delta path if
receiver-local reuse can reduce transferred bytes. If capability negotiation
fails or reuse is not available, mantle MUST fall back to ordinary substitution
behavior.

#### Scenario: Delta-capable cache hit reuses local content

- GIVEN a trusted remote cache that supports delta transfer
- AND the receiver already has reusable blob chunks for the requested output
- WHEN mantle requests that output from the cache
- THEN mantle may fetch only the missing content instead of the whole artifact
- AND a successful result is reported as a normal substitution cache hit

#### Scenario: Delta-capable HTTP cache uses the existing authority

- GIVEN a trusted HTTP cache that supports both ordinary substitution and delta
  transfer
- WHEN mantle negotiates a delta-capable fetch from that cache
- THEN the delta negotiation and stream requests go to the same cache authority
  as the ordinary substitution request

#### Scenario: Legacy cache falls back to full-artifact fetch

- GIVEN a trusted remote cache that does not support delta transfer
- WHEN mantle requests that output from the cache
- THEN mantle uses the existing full-artifact substitution path
- AND the request does not fail merely because delta support is absent

### Requirement: Store push to directory

The store layer MUST provide a library function that exports selected signed
PathInfo entries to a flat Nix binary cache directory layout.

For each selected PathInfo the function MUST:
1. Render the NAR archive from the castore node using `write_nar`
2. Write the NAR to `<dest>/nar/<nar-sha256-nixbase32>.nar`
3. Construct a `NarInfo` with correct `URL`, `FileHash`, and `FileSize` fields
4. Write the narinfo to `<dest>/<store-path-hash>.narinfo`
5. Write `<dest>/nix-cache-info` if it does not already exist

The function MUST return a structured `PushReport` with counts of pushed paths,
skipped-unsigned paths, skipped-already-present paths, and total bytes written.

#### Scenario: Push a single signed path

- GIVEN a local store with one signed PathInfo for path `hello`
- AND a writable empty target directory
- WHEN `export_paths_to_cache_dir([hello], dest)` is called
- THEN `<dest>/<digest>.narinfo` exists and is parseable as a valid narinfo
- AND `<dest>/nar/<nar-hash>.nar` exists and its sha256 matches the narinfo `NarHash`
- AND `<dest>/nix-cache-info` exists with the correct `StoreDir`
- AND the report shows `pushed_count = 1`

#### Scenario: Push skips unsigned PathInfo by default

- GIVEN a local store with one unsigned PathInfo for path `unsigned-pkg`
- AND `trust_unsigned` is false
- WHEN `export_paths_to_cache_dir([unsigned-pkg], dest)` is called
- THEN no narinfo or NAR file is written for `unsigned-pkg`
- AND the report shows `skipped_unsigned_count = 1`

#### Scenario: Push includes unsigned PathInfo when trust_unsigned is set

- GIVEN a local store with one unsigned PathInfo for path `unsigned-pkg`
- AND `trust_unsigned` is true
- WHEN `export_paths_to_cache_dir([unsigned-pkg], dest)` is called
- THEN `<dest>/<digest>.narinfo` is written for `unsigned-pkg`
- AND the report shows `pushed_count = 1`

#### Scenario: Idempotent push skips already-present paths

- GIVEN a target directory that already contains `<digest>.narinfo` for path `hello`
- WHEN `export_paths_to_cache_dir([hello], dest)` is called again
- THEN the existing narinfo is not overwritten
- AND no NAR is re-rendered for `hello`
- AND the report shows `skipped_already_present_count = 1`

#### Scenario: Push multiple paths

- GIVEN a local store with three signed PathInfo entries
- AND a writable empty target directory
- WHEN `export_paths_to_cache_dir([a, b, c], dest)` is called
- THEN three narinfo files and up to three NAR files exist in the target
- AND the report shows `pushed_count = 3`

#### Scenario: Narinfo references match stored PathInfo

- GIVEN a signed PathInfo with two runtime references
- WHEN it is pushed to a directory
- THEN the narinfo `References` field lists both referenced store path names
- AND the narinfo `NarHash` and `NarSize` match the PathInfo values

### Requirement: NAR rendering as a public StoreHandle method

`StoreHandle` MUST expose a public method for rendering a NAR archive from a
castore node to an arbitrary `AsyncWrite` sink.

This method MUST stream the NAR without buffering the full archive in memory.

#### Scenario: Render NAR to a file

- GIVEN a castore node representing a directory tree
- AND a writable file handle
- WHEN `store.render_nar(node, file)` is called
- THEN the file contains a valid NAR archive of that tree
- AND the NAR sha256 matches what `PathInfo.nar_sha256` records

#### Scenario: Render NAR to a hasher

- GIVEN a castore node
- WHEN `store.render_nar(node, hasher_writer)` is called
- THEN the hasher receives the full NAR byte stream without intermediate files

### Requirement: CLI store push subcommand

The CLI MUST provide `mantle store push` for exporting build results to a
binary cache directory.

Required arguments:
- `--to <path>` — target directory (required)
- `<path>...` — store paths to push (optional if `--all` is given)
- `--all` — push all signed paths in the local store
- `--signing-key <path>` — signing key file (for narinfo signatures)
- `--trust-unsigned` — include unsigned PathInfo entries

The command MUST use the store mutation lock to prevent concurrent push and
GC operations.

#### Scenario: Push named paths to directory

- GIVEN a successful build of `hello` and `world`
- WHEN `mantle store push --to /srv/cache hello world` runs
- THEN both paths are exported to `/srv/cache/`
- AND the command prints a summary of pushed paths and bytes

#### Scenario: Push all paths

- GIVEN a local store with five signed paths
- WHEN `mantle store push --all --to /srv/cache` runs
- THEN all five paths are exported

#### Scenario: Push with no signed paths warns

- GIVEN a local store with only unsigned PathInfo entries
- AND `--trust-unsigned` is not passed
- WHEN `mantle store push --all --to /srv/cache` runs
- THEN the command prints a warning that no paths were pushed
- AND it exits with code 0

#### Scenario: Push to nonexistent directory

- GIVEN `--to /nonexistent/dir`
- WHEN `mantle store push --all --to /nonexistent/dir` runs
- THEN the command creates the directory and its `nar/` subdirectory
- AND proceeds with the push

### Requirement: nix-cache-info file

The push operation MUST write a `nix-cache-info` file in the target directory
if one does not already exist. The file MUST contain at least `StoreDir` set
to the store prefix used by this mantle instance.

#### Scenario: nix-cache-info reflects store prefix

- GIVEN mantle running with default store prefix `/mantle/store`
- WHEN paths are pushed to an empty directory
- THEN `nix-cache-info` contains `StoreDir: /mantle/store`

#### Scenario: Existing nix-cache-info is preserved

- GIVEN a target directory with an existing `nix-cache-info`
- WHEN paths are pushed
- THEN the existing `nix-cache-info` is not modified

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

- GIVEN a cache directory produced by `mantle store push` with one signed path
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
- AND the local store uses prefix `/mantle/store`
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

The CLI `mantle store pull` `--from` argument MUST accept both local directory
ID: binary.cache.cli.storepull
paths and `http://` or `https://` URLs.

When `--from` is a URL:
- The HTTP pull path MUST be used for `http://` and `https://` URLs only
- Unsupported URL schemes such as `file://` MUST be rejected with a clear
  error before dispatch
- Cache URLs with userinfo (`user@host` or `user:pass@host`) MUST be rejected
  with a clear error before dispatch
- Phase 1 HTTP pull MUST support only public caches and MUST NOT add auth-
  header, token, insecure, or TLS-override behavior
- `--all` MUST be rejected with a clear error ("--all is not supported for
  HTTP caches; specify paths explicitly")
- Path selectors are required

When `--from` is a local path:
- Behavior is unchanged from the directory pull implementation

#### Scenario: Pull from HTTP URL
ID: binary.cache.cli.storepull.httpurl

- GIVEN `--from https://cache.example.com`
- AND store path arguments are provided
- WHEN `mantle store pull` runs
- THEN paths are fetched over HTTP

#### Scenario: Pull --all from HTTP is rejected
ID: binary.cache.cli.storepull.httpallrejected

- GIVEN `--from https://cache.example.com`
- AND `--all` is passed
- WHEN `mantle store pull` runs
- THEN the command fails with an error message explaining that --all is not
  supported for HTTP caches

#### Scenario: Pull from HTTP requires explicit path selectors
ID: binary.cache.cli.storepull.httprequirespaths

- GIVEN `--from https://cache.example.com`
- AND no store path arguments are provided
- AND `--all` is not passed
- WHEN `mantle store pull` runs
- THEN the command fails before any HTTP request is issued
- AND the error explains that HTTP pull requires explicit store path selectors

#### Scenario: Pull rejects unsupported URL schemes
ID: binary.cache.cli.storepull.unsupportedurlscheme

- GIVEN `--from file://cache.example.com`
- WHEN `mantle store pull` runs
- THEN the command fails before local-path dispatch or any HTTP request
- AND the error explains that only `http://` and `https://` URLs are supported

#### Scenario: Pull rejects HTTP URL userinfo
ID: binary.cache.cli.storepull.userinforejected

- GIVEN `--from https://user@cache.example.com`
- WHEN `mantle store pull` runs
- THEN the command fails before any HTTP request is issued
- AND the error explains that HTTP pull URLs must not include credentials

#### Scenario: Pull imports from a pushed on-disk HTTP cache into a fresh store
ID: binary.cache.cli.storepull.httpfreshstateroundtrip

- GIVEN a pushed on-disk cache served over local HTTP
- AND a fresh local store/state directory plus the source verifying key
- WHEN `mantle store pull --from http://127.0.0.1:PORT <logical-path>` runs
- THEN the requested path is imported into the fresh store
- AND the output exists on disk after the command succeeds

### Requirement: Store pull from HTTP cache

The store layer MUST provide a library function that imports PathInfo entries
from a remote HTTP/HTTPS Nix binary cache into the local store.
ID: binary.cache.storepull.http

For each requested store path the function MUST:
1. After mandatory `nix-cache-info` preflight, check local PathInfo and skip
   the path without any narinfo or NAR request if the requested store path
   already exists locally
2. Construct the narinfo URL as `<cache-url>/<nixbase32(digest)>.narinfo`
3. Fetch and parse the narinfo, verifying at least one signature against
   trusted public keys (unless `trust_unsigned` is set)
4. Validate that the narinfo `StorePath` matches the requested store path
   exactly and still uses the local store directory prefix before any NAR
   download or PathInfo persistence
5. Resolve the NAR URL from the narinfo `URL` field relative to the cache URL.
   Absolute URLs, root-relative paths, authority-changing values, and any
   relative path that escapes the normalized cache base path MUST be rejected;
   only relative paths that stay under the normalized cache base path (for
   example `nar/<hash>.nar.xz`) are resolved
6. After `nix-cache-info` preflight handling has completed, HTTP pull MUST NOT
   follow redirects for narinfo or NAR requests after the cache base URL and
   relative NAR path have been resolved
7. Download the NAR, treating absent or `none` `Compression` as uncompressed
   input, decompressing when the field specifies `bzip2`, `gzip`, `xz`, or
   `zstd`, and rejecting unknown compression labels with
   `skipped_parse_error_count`
8. Ingest the NAR into castore via `ingest_nar_and_hash`, verifying the
   NAR hash matches the narinfo `NarHash`
9. Parse narinfo `References` as store-path basenames under the local store
   directory prefix, reject malformed or absolute/differently-prefixed
   reference entries before persistence, and retain the validated reference
   list unchanged in the imported PathInfo
10. Construct and persist a PathInfo from the narinfo fields
11. Export the castore node to the local store directory on disk

The cache base URL MUST be normalized by dropping query and fragment
components so `https://host/cache`, `https://host/cache/`, and
`https://host/cache?ignored=1#frag` resolve `nix-cache-info`, narinfo, and
relative NAR URLs under the same `/cache/` prefix. Cache URLs with userinfo
(credentials in `user@host` or `user:pass@host` form) MUST be rejected.

The function MUST return the same structured `PullReport` type used by
directory pull. HTTP-specific skip reasons MUST map to existing report fields:
already-present local PathInfo → `skipped_already_present_count`, untrusted or
unsigned narinfo with `trust_unsigned = false` → `skipped_untrusted_count`,
narinfo 404/403 → `skipped_missing_nar_count`, other non-success non-redirect
narinfo HTTP statuses (including 400/401/410/5xx) →
`skipped_parse_error_count`, NAR non-success non-redirect HTTP statuses
(including 401/403/404/410/5xx) → `skipped_missing_nar_count`, redirect
rejection for narinfo/NAR requests → `skipped_parse_error_count`, network or
transport error while fetching narinfo/NAR → `skipped_parse_error_count`,
narinfo parse failure or requested store-path mismatch →
`skipped_parse_error_count`, narinfo store-directory prefix mismatch →
`skipped_store_dir_mismatch_count`, malformed or absolute/differently-prefixed
reference entries → `skipped_parse_error_count`, decompression failure or
`NarHash` mismatch → `skipped_hash_mismatch_count`.

Any outcome mapped to a skip count MUST skip only the current path and allow
remaining requested paths to continue importing.

PathInfo persistence failure or export failure MUST fail the whole call instead
of returning a skip count, because the path was already accepted as valid and
cannot be reported as a partial import.

Phase 1 imports only the explicitly requested paths. Missing references
(narinfo `References` entries not in the local store) are NOT fetched
recursively.

#### Scenario: Pull a single path from HTTP cache
ID: binary.cache.storepull.http.happypath

- GIVEN a remote HTTP cache at `http://cache.example.com` with one signed path
- AND the local store has no PathInfo for that path
- AND the narinfo signature matches a trusted public key
- WHEN `import_paths_from_http_cache(handle, url, [path], options)` is called
- THEN the narinfo is fetched from `http://cache.example.com/<digest>.narinfo`
- AND the NAR is downloaded and ingested into castore
- AND a PathInfo is persisted in the local store
- AND the report shows `imported_count = 1`

#### Scenario: Pull skips 404/403 narinfo paths
ID: binary.cache.storepull.http.narinfo404

- GIVEN a remote HTTP cache that returns 404 or 403 for `<digest>.narinfo`
- WHEN `import_paths_from_http_cache` is called for that path
- THEN the path is skipped
- AND the report shows `skipped_missing_nar_count = 1`

#### Scenario: Pull maps non-success narinfo HTTP statuses to parse-error count
ID: binary.cache.storepull.http.narinfo5xx

- GIVEN a remote HTTP cache that returns a non-success non-redirect HTTP status other than 403/404 for `<digest>.narinfo`
- WHEN `import_paths_from_http_cache` is called for that path
- THEN the path is skipped
- AND the report shows `skipped_parse_error_count = 1`

#### Scenario: Pull maps narinfo transport failure to parse-error count
ID: binary.cache.storepull.http.narinfotransportfailure

- GIVEN a remote HTTP cache whose `<digest>.narinfo` request fails with a timeout, reset, TLS failure, or other transport error
- WHEN `import_paths_from_http_cache` is called for that path
- THEN the path is skipped
- AND the report shows `skipped_parse_error_count = 1`
- AND remaining requested paths continue importing

#### Scenario: Pull handles compressed NAR
ID: binary.cache.storepull.http.compressednar

- GIVEN a remote HTTP cache serving xz-compressed NARs
- AND the narinfo `Compression` field is `xz`
- WHEN `import_paths_from_http_cache` is called
- THEN the NAR is decompressed during ingestion
- AND the decompressed NAR hash matches the narinfo `NarHash`

#### Scenario: Pull handles the full advertised compression set
ID: binary.cache.storepull.http.allcompressiontags

- GIVEN remote HTTP cache entries whose narinfo `Compression` field is `bzip2`, `gzip`, `xz`, or `zstd`
- WHEN `import_paths_from_http_cache` is called for each of those entries
- THEN each supported label is accepted by the decoder dispatch
- AND each entry is decompressed and validated through the same import path

#### Scenario: Pull accepts uncompressed NAR input
ID: binary.cache.storepull.http.uncompressednar

- GIVEN a remote HTTP cache entry whose narinfo omits `Compression` or sets it to `none`
- WHEN `import_paths_from_http_cache` is called for that entry
- THEN the NAR is read without decompression
- AND the import proceeds through the normal validation path

#### Scenario: Pull rejects unknown compression labels
ID: binary.cache.storepull.http.unknowncompression

- GIVEN a remote HTTP cache entry whose narinfo `Compression` field uses an unknown label
- WHEN `import_paths_from_http_cache` is called for that entry
- THEN the path is skipped before ingestion
- AND the report shows `skipped_parse_error_count = 1`

#### Scenario: Pull rejects untrusted signatures over HTTP
ID: binary.cache.storepull.http.rejectuntrustedsignatures

- GIVEN a remote cache with narinfos signed by an unknown key
- AND `trust_unsigned` is false
- WHEN `import_paths_from_http_cache` is called
- THEN no NAR is downloaded
- AND the report shows `skipped_untrusted_count = 1`

#### Scenario: Pull rejects unsigned narinfo when trust-unsigned is false
ID: binary.cache.storepull.http.rejectsunsignedwhenstrict

- GIVEN a remote cache with an unsigned narinfo
- AND `trust_unsigned` is false
- WHEN `import_paths_from_http_cache` is called
- THEN no NAR is downloaded
- AND the report shows `skipped_untrusted_count = 1`

#### Scenario: Pull accepts unsigned narinfo when trust-unsigned is set
ID: binary.cache.storepull.http.acceptsuntrustedwhenallowed

- GIVEN a remote cache with an unsigned or otherwise untrusted narinfo
- AND `trust_unsigned` is true
- WHEN `import_paths_from_http_cache` is called
- THEN the path is imported if the remaining validation steps succeed
- AND the report shows `imported_count = 1`

#### Scenario: Network error during HTTP pull
ID: binary.cache.storepull.http.networkerrorcontinues

- GIVEN a remote cache that is unreachable
- WHEN `import_paths_from_http_cache` is called for a path
- THEN the path is skipped with a warning
- AND the pull continues for remaining paths

#### Scenario: Pull skips already-present local PathInfo without fetching narinfo
ID: binary.cache.storepull.http.alreadypresentshortcircuit

- GIVEN a requested store path that already exists in the local PathInfo service
- WHEN `import_paths_from_http_cache` is called for that path
- THEN `/nix-cache-info` is still requested for that cache
- AND no narinfo request is issued for that digest
- AND no NAR request is issued for that digest
- AND the report shows `skipped_already_present_count = 1`

#### Scenario: Pull rejects absolute or escaping narinfo URL values
ID: binary.cache.storepull.http.absoluteurlrejected

- GIVEN a remote HTTP cache whose narinfo `URL` field is absolute, root-relative, authority-changing, or escapes the normalized cache base path
- WHEN `import_paths_from_http_cache` is called for that path
- THEN the path is skipped before any NAR download begins
- AND the report shows `skipped_parse_error_count = 1`

#### Scenario: Pull does not follow narinfo or NAR redirects
ID: binary.cache.storepull.http.redirectblocked

- GIVEN a resolved narinfo or relative NAR request that responds with a redirect
- WHEN `import_paths_from_http_cache` performs that request
- THEN the redirect is not followed
- AND the path is skipped rather than fetching data from the redirected location
- AND the report shows `skipped_parse_error_count = 1`

#### Scenario: Pull does not recurse into missing references
ID: binary.cache.storepull.http.nonrecursivemissingreferences

- GIVEN a requested HTTP path whose narinfo `References` include store paths not present locally
- WHEN `import_paths_from_http_cache` is called for only that requested path
- THEN only the requested path is fetched over HTTP
- AND no narinfo request is issued for the missing reference digests
- AND the imported PathInfo retains its reference list unchanged

#### Scenario: Pull parses references with the local store prefix
ID: binary.cache.storepull.http.referenceprefixparsing

- GIVEN a local store using a non-default logical store prefix
- AND a fetched narinfo whose `References` entries are valid store-path basenames for that same local store
- WHEN `import_paths_from_http_cache` imports the path
- THEN the reference entries are accepted and retained unchanged in the imported PathInfo

#### Scenario: Pull rejects malformed or mismatched references
ID: binary.cache.storepull.http.referencemismatch

- GIVEN a fetched narinfo whose `References` entries are malformed or are encoded as absolute paths using the local or a different store-directory prefix
- WHEN `import_paths_from_http_cache` imports the path
- THEN the path is skipped before persistence
- AND the report records `skipped_parse_error_count = 1`

#### Scenario: Pull rejects malformed narinfo text
ID: binary.cache.storepull.http.narinfoparsefailure

- GIVEN a remote HTTP cache that returns malformed narinfo text for `<digest>.narinfo`
- WHEN `import_paths_from_http_cache` is called for that path
- THEN the path is skipped before NAR download
- AND the report shows `skipped_parse_error_count = 1`

#### Scenario: Pull rejects narinfo store-path mismatch
ID: binary.cache.storepull.http.storepathmismatch

- GIVEN a requested store path whose narinfo parses under the local store prefix
- AND the narinfo `StorePath` names a different store path than the requested one
- WHEN `import_paths_from_http_cache` is called for that requested path
- THEN the path is skipped before NAR download
- AND the report shows `skipped_parse_error_count = 1`

#### Scenario: Pull rejects narinfo store-path prefix mismatch
ID: binary.cache.storepull.http.storepathprefixmismatch

- GIVEN a remote cache whose `nix-cache-info` `StoreDir` matches the local store prefix
- AND a fetched narinfo whose `StorePath` uses a different store-directory prefix
- WHEN `import_paths_from_http_cache` is called for that requested path
- THEN the path is skipped before NAR download
- AND the report shows `skipped_store_dir_mismatch_count = 1`

#### Scenario: Pull maps NAR HTTP failure to missing-nar count
ID: binary.cache.storepull.http.nardownloadfailure

- GIVEN a fetched narinfo whose relative `URL` points at a missing NAR object or any non-success non-redirect HTTP NAR response
- WHEN `import_paths_from_http_cache` downloads that NAR
- THEN the path is skipped
- AND the report shows `skipped_missing_nar_count = 1`

#### Scenario: Pull maps NAR transport failure to parse-error count
ID: binary.cache.storepull.http.nartransportfailure

- GIVEN a fetched narinfo whose NAR request fails with a timeout, reset, TLS failure, or other transport error
- WHEN `import_paths_from_http_cache` downloads that NAR
- THEN the path is skipped
- AND the report shows `skipped_parse_error_count = 1`

#### Scenario: Pull maps decompression or NarHash failure to hash-mismatch count
ID: binary.cache.storepull.http.narhashmismatch

- GIVEN a fetched narinfo whose NAR bytes decompress incorrectly or do not match `NarHash`
- WHEN `import_paths_from_http_cache` ingests that NAR
- THEN the path is skipped
- AND the report shows `skipped_hash_mismatch_count = 1`

#### Scenario: Cache base URL path prefix is stable
ID: binary.cache.storepull.http.baseurlnormalization

- GIVEN `--from https://host/cache` or `--from https://host/cache/`
- WHEN the pull function resolves `nix-cache-info`, narinfo, and relative NAR URLs
- THEN every request stays under the `/cache/` prefix
- AND query/fragment components on the base URL do not affect the resolved remote objects
- AND both forms resolve the same remote objects

#### Scenario: Library rejects cache URL userinfo
ID: binary.cache.storepull.http.cacheurluserinfo

- GIVEN `import_paths_from_http_cache(...)` receives `https://user@host/cache` or `https://user:pass@host/cache`
- WHEN the function begins
- THEN it returns an error before any HTTP request is issued
- AND the error explains that HTTP pull URLs must not include credentials

#### Scenario: PathInfo persistence failure aborts the whole call
ID: binary.cache.storepull.http.persistencefailure

- GIVEN a path whose narinfo and NAR both validate successfully
- AND `pathinfo_service.put()` fails for that path
- WHEN `import_paths_from_http_cache` persists the accepted PathInfo
- THEN the function returns an error for the whole call
- AND the path is not reported as a skipped partial import

#### Scenario: Export failure aborts the whole call
ID: binary.cache.storepull.http.exportfailure

- GIVEN a path whose narinfo and NAR both validate successfully
- AND PathInfo persistence succeeds
- AND export to the local store directory fails
- WHEN `import_paths_from_http_cache` exports the accepted path
- THEN the function returns an error for the whole call
- AND the path is not reported as a skipped partial import

### Requirement: Remote nix-cache-info validation

Before importing any paths over HTTP, the pull function MUST fetch
`<cache-url>/nix-cache-info` and validate that the remote `StoreDir` matches
the local store directory prefix.
ID: binary.cache.remotenixcacheinfo.validation

The function MUST perform this `nix-cache-info` preflight even when every
requested store path is already present locally; the already-present short-
circuit only forbids narinfo or NAR fetches for those paths.

If the remote `StoreDir` does not match, the function MUST return an error
without fetching any narinfos.

`nix-cache-info` redirects MUST NOT be followed; redirect rejection is treated
as the same warning-path preflight failure as other request failures.

If `nix-cache-info` is absent (404), unparseable, or fails due to another
non-success non-redirect HTTP status (including 400/401/403/410/5xx),
transport error, or redirect rejection, the function MUST proceed with a
warning (some caches omit this file). That warning is observed through the
existing logging surface (`tracing::warn!` / CLI stderr), not via a new
`PullReport` field.

#### Scenario: StoreDir matches
ID: binary.cache.remotenixcacheinfo.validation.storedirmatch

- GIVEN a remote cache with `nix-cache-info` containing `StoreDir: /nix/store`
- AND the local store uses prefix `/nix/store`
- WHEN HTTP pull begins
- THEN narinfo fetching proceeds normally

#### Scenario: StoreDir mismatch is a hard error
ID: binary.cache.remotenixcacheinfo.validation.storedirmismatch

- GIVEN a remote cache with `nix-cache-info` containing `StoreDir: /nix/store`
- AND the local store uses prefix `/mantle/store`
- WHEN HTTP pull begins
- THEN the function returns an error before any narinfo is fetched

#### Scenario: Missing nix-cache-info proceeds with warning
ID: binary.cache.remotenixcacheinfo.validation.missingwarning

- GIVEN a remote cache that returns 404 for `/nix-cache-info`
- WHEN HTTP pull begins
- THEN a warning is logged
- AND narinfo fetching proceeds assuming the remote StoreDir matches the local
  prefix (narinfo-level store-path validation will catch actual mismatches)

#### Scenario: Malformed nix-cache-info proceeds with warning
ID: binary.cache.remotenixcacheinfo.validation.malformedwarning

- GIVEN a remote cache that serves `/nix-cache-info` without a parseable `StoreDir:` entry
- WHEN HTTP pull begins
- THEN a warning is logged
- AND narinfo fetching proceeds assuming the remote StoreDir matches the local prefix

#### Scenario: nix-cache-info request failure proceeds with warning
ID: binary.cache.remotenixcacheinfo.validation.requestfailurewarning

- GIVEN a remote cache whose `/nix-cache-info` request fails with a timeout, connection refusal, TLS failure, or a non-success non-redirect HTTP status such as 401, 410, or 5xx
- WHEN HTTP pull begins
- THEN a warning is logged
- AND narinfo fetching proceeds assuming the remote StoreDir matches the local prefix

#### Scenario: nix-cache-info redirect rejection proceeds with warning
ID: binary.cache.remotenixcacheinfo.validation.redirectwarning

- GIVEN a remote cache whose `/nix-cache-info` request responds with a redirect
- WHEN HTTP pull begins
- THEN a warning is logged
- AND narinfo fetching proceeds assuming the remote StoreDir matches the local prefix

#### Scenario: nix-cache-info request failure still rejects narinfo prefix mismatch
ID: binary.cache.remotenixcacheinfo.validation.requestfailureprefixmismatch

- GIVEN a remote cache whose `/nix-cache-info` request fails with a timeout, connection refusal, TLS failure, or a non-success non-redirect HTTP status such as 401, 410, or 5xx
- AND the fetched narinfo `StorePath` uses a different store directory prefix than the local store
- WHEN HTTP pull begins for that requested path
- THEN the path is skipped before NAR download
- AND the report records a store-directory mismatch outcome

#### Scenario: Missing nix-cache-info still rejects narinfo prefix mismatch
ID: binary.cache.remotenixcacheinfo.validation.missingprefixmismatch

- GIVEN a remote cache that returns 404 for `/nix-cache-info`
- AND the fetched narinfo `StorePath` uses a different store directory prefix than the local store
- WHEN HTTP pull begins for that requested path
- THEN the path is skipped before NAR download
- AND the report records a store-directory mismatch outcome

#### Scenario: Malformed nix-cache-info still rejects narinfo prefix mismatch
ID: binary.cache.remotenixcacheinfo.validation.malformedprefixmismatch

- GIVEN a remote cache that serves malformed `/nix-cache-info`
- AND the fetched narinfo `StorePath` uses a different store directory prefix than the local store
- WHEN HTTP pull begins for that requested path
- THEN the path is skipped before NAR download
- AND the report records a store-directory mismatch outcome

