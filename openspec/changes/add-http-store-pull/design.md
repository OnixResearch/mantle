## Context

`crunch store pull` imports narinfo + NAR pairs from a local directory into the
local store. The HTTP extension adds the same pipeline over the network.

The existing `NixHTTPPathInfoService::get()` already does narinfo fetch →
signature verify → NAR download → decompression → `ingest_nar_and_hash()` →
PathInfo construction. The pull HTTP path reuses the same logical sequence but
drives it from the pull module rather than the substituter.

## Goals / Non-Goals

**Goals:**
- Import specific store paths from an HTTP/HTTPS binary cache URL.
- Verify narinfo signatures against trusted public keys before NAR download.
- Validate remote `StoreDir` via `nix-cache-info` before any import.
- Handle compressed NARs (xz, zstd, gzip, bzip2).
- Produce the same `PullReport` as directory pull.
- Skip paths already present in local PathInfo (idempotent).

**Non-Goals:**
- `--all` over HTTP. No standard Nix cache exposes a directory listing.
  Phase 1 requires explicit path selectors for HTTP sources.
- Push over HTTP. That's a separate upload/signing concern.
- Credential/auth for private caches. Phase 1 handles public caches only.
  Auth headers are a natural follow-up.
- Parallel downloads. Phase 1 is sequential per path. Concurrency is a
  follow-up optimization.

## Decisions

### 1. PullSource enum, not a trait

**Choice:** Add `PullSource { Directory(PathBuf), Http(Url) }` and dispatch at
the top of `cmd_store_pull`.

**Rationale:** The two transports share the same report type and options but
have different I/O. An enum is simpler than a trait object and keeps the two
paths independently testable. The CLI detects the source type by URL scheme
prefix.

### 2. Reuse reqwest + async-compression from snix-store

**Choice:** Use `reqwest::Client` for HTTP and `async_compression::tokio::bufread`
decoders, matching `NixHTTPPathInfoService`.

**Rationale:** These crates are already in the dependency tree. No new deps.
The decompression dispatch (match on narinfo `Compression` field) can be
extracted into a shared helper.

### 3. Fetch nix-cache-info first for StoreDir validation

**Choice:** Before fetching any narinfo, GET `<url>/nix-cache-info`, parse
`StoreDir`, and compare with the local store prefix.

- **Match:** proceed with narinfo fetches.
- **Mismatch:** hard `Error::Store` — abort before any narinfo is fetched.
- **Absent (404) or unparseable:** log a warning and proceed, assuming the
  remote StoreDir matches the local prefix. Narinfo-level parse failures
  (via `NarInfo::parse_with_store_dir`) will catch actual mismatches.

**Rationale:** Downloading and ingesting NARs from a cache with a different
store prefix wastes bandwidth and produces unusable PathInfos. Fail fast when
we can detect it. Some caches (e.g., S3 static hosting) omit `nix-cache-info`,
so absence is not a hard error.

### 4. Narinfo URL construction from store path digest

**Choice:** For each requested store path, construct
`<cache-url>/<nixbase32(digest)>.narinfo`, parse the response, then resolve the
NAR URL from the narinfo `URL` field.

**Rationale:** This is the standard Nix cache protocol. The digest-based lookup
avoids needing the full store path name upfront (though the path filter provides
it). It matches `NixHTTPPathInfoService::derive_narinfo_url()`.

### 5. Narinfo `URL` field: relative-only, no redirect following on NAR fetch

**Choice:** When resolving the NAR download URL from a narinfo's `URL` field,
reject any value that parses as an absolute URL (has a scheme). Only relative
paths like `nar/<hash>.nar.xz` are joined with the cache base URL.
Additionally, build the reqwest `Client` with
`redirect::Policy::limited(10)` restricted to same-scheme (no
http→file or http→ftp redirects). This closes the SSRF surface that
absolute URL injection or open-redirect chains could exploit.

**Implementation:** `url::Url::parse(url_field)` on a relative path returns
`Err` — that's the success case. If it returns `Ok` (has scheme), skip the
path with `skipped_parse_error_count += 1`.

### 6. Signature verification reuses existing pull options

**Choice:** `PullOptions` already carries `trust_unsigned: bool` and
`trusted_public_keys: Vec<VerifyingKey>`. The HTTP path reuses the same
options struct and the same verification logic as directory pull:
parse narinfo → compute fingerprint with `fingerprint_with_store_dir()` →
check each signature against each trusted key. If no match and
`!trust_unsigned`, increment `skipped_untrusted_count` and skip before
downloading the NAR.

**Rationale:** This is the same logic as `pull_single_narinfo()` step 4.
No new verification code is needed.

### 7. Path selectors resolve to digests

**Choice:** The caller provides store path strings. The pull function parses
each into a `StorePath` to extract the digest, then fetches the narinfo by
digest. If the parsed store path doesn't match the narinfo's `StorePath`, skip
with a report entry.

**Rationale:** Users copy-paste full store paths from `crunch store list` or
build output. Parsing to `StorePath` validates the format and extracts the
20-byte digest needed for the narinfo URL.

### 8. Error handling mirrors directory pull with explicit report field mapping

**Choice:** Network errors, 404s, parse failures, signature rejections, and
hash mismatches are all reported as skip counts in `PullReport`, not as
hard errors. Only `nix-cache-info` mismatch and invalid URL are hard errors.

HTTP-specific skip-reason mapping to existing `PullReport` fields:
- Narinfo 404 → `skipped_missing_nar_count`
- Narinfo parse failure → `skipped_parse_error_count`
- Network/timeout error → `skipped_parse_error_count`
- Untrusted signature → `skipped_untrusted_count`
- NAR download failure → `skipped_missing_nar_count`
- NAR hash mismatch → `skipped_hash_mismatch_count`
- Absolute URL in narinfo → `skipped_parse_error_count`

`skipped_untrusted_count` already exists in `PullReport` (used by directory
pull). No new fields are needed.

**Rationale:** A remote cache may have partial coverage. Skipping missing
paths and reporting counts lets the operator see what succeeded vs. failed
without aborting the entire pull.

### 9. Persist via pathinfo_service.put() + export to disk

**Choice:** Same as directory pull — `put()` the constructed PathInfo, then
`export_castore_to_disk()`. No `persist_and_export_signed_output()`.

**Rationale:** Consistency with directory pull. Imported paths have narinfo
signatures as trust anchors, not build provenance.

### 10. Decompression dispatch: inline, not extracted

**Choice:** Phase 1 inlines the compression-format match in
`import_paths_from_http_cache()`, duplicating the pattern from
`NixHTTPPathInfoService::get()`. Extraction into a shared helper is a
follow-up.

**Rationale:** The match is ~15 lines. Extracting it requires touching
vendored `snix-store` code and adds coupling. Inline duplication is
acceptable for Phase 1; a shared helper becomes worthwhile if a third
caller appears.

### 11. Reqwest client configuration

**Choice:**
- Connect timeout: 30 seconds
- Read timeout: 5 minutes (NARs can be large)
- Redirect policy: limited to 10 hops, same-scheme only
- User-agent: `crunch/<version>`
- No maximum response size guard in Phase 1 (NARs are hash-verified,
  so a malicious oversize response is caught at ingestion or wastes
  bandwidth but does not corrupt state)

**Rationale:** Matches typical HTTP client defaults. The timeouts prevent
hangs against misbehaving servers. Same-scheme redirect restriction closes
the SSRF gap noted in the spec.

## Verification Strategy

| Spec scenario | Test type | Method |
|---|---|---|
| Pull single path from HTTP cache | Unit | Push to dir, serve with `TcpListener` + manual HTTP response, pull via `import_paths_from_http_cache` |
| Pull skips 404 paths | Unit | HTTP server returns 404 for narinfo request |
| Pull handles compressed NAR | Unit | Push to dir, xz-compress the NAR, update narinfo `Compression`/`FileHash`/`FileSize`, serve, pull |
| Pull rejects untrusted signatures | Unit | Serve narinfo signed with unknown key, verify `skipped_untrusted_count` |
| Network error during HTTP pull | Unit | Server drops connection mid-response |
| StoreDir mismatch | Unit | Serve `nix-cache-info` with wrong StoreDir |
| Missing nix-cache-info | Unit | Server returns 404 for `/nix-cache-info`, pull proceeds |
| --all rejection over HTTP | Integration | CLI returns error for `--all --from https://...` |
| Push → HTTP-serve → pull round-trip | Integration | Build, push, serve with real HTTP fixture, pull from URL, verify PathInfo + on-disk output |

All HTTP tests use a real `TcpListener` on `127.0.0.1:0` serving actual
narinfo/NAR content from a pushed directory — no in-memory mocks.

## Risks / Trade-offs

**[No --all over HTTP]** Users must specify paths explicitly. This is a
real limitation for "sync everything from remote". Mitigation: `--all` is
documented as directory-only. A `--closure` follow-up could walk references.

**[No auth]** Private caches (S3 with IAM, Cachix with tokens) require
auth headers. Phase 1 handles only public caches. Mitigation: document
the limitation; auth is an additive follow-up.

**[Sequential downloads]** Each path is fetched serially. For large imports
this is slow. Mitigation: the core function is async and can be wrapped in
`futures::stream::iter(...).buffer_unordered(n)` later without API changes.

**[Orphaned castore on hash mismatch]** Same as directory pull and
substitution — orphaned blobs await GC.

**[Redirect-based SSRF]** reqwest follows redirects by default. Mitigated
by same-scheme restriction and relative-URL-only narinfo resolution.
