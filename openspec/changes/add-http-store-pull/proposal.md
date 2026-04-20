## Why

`crunch store pull` currently only accepts `--from <local-directory>`. This
forces users to mount or copy remote caches to the local filesystem before
importing. The primary use case for binary cache sharing — push to an HTTP-
accessible store (S3, nginx, Cloudflare R2), pull from another machine over
the network — requires HTTP transport.

All required primitives exist:

- `NixHTTPPathInfoService` already fetches narinfo text, verifies signatures,
  downloads + decompresses NARs, ingests into castore, and constructs PathInfo.
  It handles bzip2, gzip, xz, and zstd decompression.
- `import_paths_from_cache_dir()` already has the verify → ingest → persist →
  export pipeline. The gap is only the source transport (filesystem reads vs
  HTTP GETs).
- `nix-cache-info` parsing for remote `StoreDir` validation is straightforward.

The substituter path (`--substituters`) does transparent background
substitution during builds. `crunch store pull --from <url>` is a different
user intent: explicit, selective, operator-directed import of known paths into
the local store. Keeping them separate avoids coupling two distinct workflows.

## Non-Goals (Phase 1)

- **Recursive closure pull.** Phase 1 imports only the explicitly listed paths.
  Missing references are not fetched automatically. A `--with-closure` flag is
  a natural follow-up.
- **`--all` over HTTP.** No standard Nix cache exposes a directory listing.
- **Authenticated caches.** Private caches (S3 IAM, Cachix tokens) require
  auth headers. Phase 1 handles public caches only.
- **`file://` URL scheme.** Local paths are already handled by directory pull.
- **Substituter coupling.** `store pull` is explicit operator import, not
  transparent background substitution.
- **Parallel downloads.** Phase 1 is sequential per path.
- **TLS certificate overrides.** Phase 1 uses reqwest defaults (system CA
  store). No `--insecure` flag.

## What Changes

- **`crunch_store::pull` module.** `PullSource` enum distinguishes local
  directory vs HTTP URL. `import_paths_from_http_cache()` fetches narinfo by
  store-path digest, verifies signatures, downloads + decompresses the NAR,
  ingests into castore, persists PathInfo, and exports to disk.
- **`--from` accepts URLs.** The CLI detects `http://` or `https://` prefixes
  and routes to the HTTP pull path. Local directory pull is unchanged.
- **Remote `nix-cache-info` validation.** Before pulling, the HTTP path
  fetches `nix-cache-info` from the remote and validates that its `StoreDir`
  matches the local store prefix. Mismatch is a hard error.
- **Compression support.** HTTP pull handles compressed NARs (xz, zstd, gzip,
  bzip2) by reusing the same decompression logic as `NixHTTPPathInfoService`.
  Local directory pull remains uncompressed-only (matching push output).
- **`--all` over HTTP.** Not supported in Phase 1 — HTTP caches have no
  standard directory listing. `--all` with an HTTP URL returns a clear error
  directing the user to specify paths explicitly.

## Capabilities

### New Capabilities
- `store-pull-http`: import PathInfo entries from a remote HTTP/HTTPS Nix
  binary cache into the local store.
- `store-pull-http-compression`: decompress bzip2/gzip/xz/zstd NARs fetched
  from remote HTTP caches.

### Modified Capabilities
- `cli-store-pull`: `--from` now accepts `http://` and `https://` URLs in
  addition to local directory paths.

## Impact

- **Files**: `crates/crunch-store/src/pull.rs` (extend with HTTP path),
  `src/store_cmd.rs` (URL detection in `--from`), `src/main.rs` (no schema
  change — `--from` stays a string, detection is at dispatch).
- **APIs**: `import_paths_from_http_cache()` as a new library entry point
  alongside `import_paths_from_cache_dir()`. `PullSource` enum as a public
  type.
- **Dependencies**: `reqwest` (already in tree via snix-store). Async
  compression crates (already in tree via snix-store).
- **Testing**: unit test with local HTTP server serving pushed cache;
  integration test for push → HTTP-serve → pull round-trip; test for
  store-prefix mismatch via remote `nix-cache-info`; test for compressed NAR
  fetch; test for `--all` rejection over HTTP.
