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

When the source is HTTP, `cmd_store_pull` classifies `--from` by exact string
shape: `http://...` and `https://...` are parsed as HTTP URL candidates,
other `scheme://...` inputs are rejected immediately as unsupported URL
schemes, and only strings without `://` are treated as local filesystem paths.
For HTTP candidates, `cmd_store_pull` rejects `--all` before any other HTTP
validation so the operator gets the dedicated `--all is not supported for HTTP
caches; specify paths explicitly` error. It also rejects cache URLs with
userinfo before dispatch because Phase 1 supports only public caches. If
`--all` is not set, it then rejects zero explicit path selectors before
opening the client or fetching `nix-cache-info`. HTTP pull stays an
operator-directed import of named paths rather than a cache listing workflow.

**Rationale:** The two transports share the same report type and options but
have different I/O. An enum is simpler than a trait object and keeps the two
paths independently testable. The explicit `://` classification rule prevents
unsupported schemes from falling through to local-path handling.

### 2. Reuse reqwest + async-compression from snix-store

**Choice:** Use `reqwest::Client` for HTTP and `async_compression::tokio::bufread`
decoders, matching `NixHTTPPathInfoService`.

**Rationale:** These crates are already in the dependency tree. No new deps.
The decompression dispatch (match on narinfo `Compression` field) can be
extracted into a shared helper.

### 3. Fetch nix-cache-info first for StoreDir validation

**Choice:** Before any per-path already-present short-circuit or narinfo
fetch, GET `<url>/nix-cache-info`, parse `StoreDir`, and compare with the
local store prefix.

- **Match:** proceed with narinfo fetches.
- **Mismatch:** hard `Error::Store` — abort before any narinfo is fetched.
- **Absent (404), unparseable, redirect-rejected, transport failure, or other
  non-success non-redirect HTTP status (including 400/401/403/410/5xx):** log
  a warning and proceed, assuming the remote StoreDir matches the local
  prefix. Narinfo-level store-prefix mismatches are still routed specifically
  to `skipped_store_dir_mismatch_count` by detecting
  `NarInfo::parse_with_store_dir` `InvalidStorePath` failures separately from
  generic parse errors.

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

### 5. Narinfo `URL` field: relative-only, no redirects

**Choice:** When resolving the NAR download URL from a narinfo's `URL` field,
reject any value that parses as an absolute URL (has a scheme). Only relative
paths like `nar/<hash>.nar.xz` are joined with the cache base URL.
Additionally, build the reqwest `Client` with `redirect::Policy::none()` so
`nix-cache-info`, narinfo, and NAR requests never follow redirects. This
closes the SSRF surface that absolute URL injection or open-redirect chains
could exploit.

**Implementation:** `url::Url::parse(url_field)` on a relative path returns
`Err` — that's the success case. If it returns `Ok` (has scheme), or if the
value starts with `/` or `//`, reject it as non-relative and skip the path
with `skipped_parse_error_count += 1`. After joining a candidate relative path,
verify that the resolved URL still stays under the normalized cache-base path;
reject dot-segment or traversal forms that escape that prefix. All HTTP
requests use the same redirect-disabled client.

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
 each into a `StorePath` to extract the digest, checks local PathInfo first,
 then fetches the narinfo by digest. If the parsed store path doesn't match the
 narinfo's `StorePath`, skip with `skipped_parse_error_count += 1`.

**Rationale:** Users copy-paste full store paths from `crunch store list` or
build output. Parsing to `StorePath` validates the format and extracts the
20-byte digest needed for the narinfo URL.

### 8. Reference parsing uses the local store prefix

**Choice:** Parse narinfo `References` as store-path basenames under the same
local store prefix used for narinfo `StorePath` validation before constructing
the imported `PathInfo`. Retain the validated reference list unchanged on
success. If a reference entry is malformed, or is encoded as an absolute path
using the local or a different store-directory prefix, skip with
`skipped_parse_error_count += 1`.

**Rationale:** Narinfo `References` are basename entries, not full absolute
store paths. Treating absolute or differently-prefixed reference text as
malformed keeps the runtime behavior aligned with the wire format while still
forcing non-default local store prefixes through the same accepted-object path.

### 9. Error handling mirrors directory pull with explicit report field mapping

**Choice:** Network errors, 404s, parse failures, signature rejections, and
hash mismatches are all reported as skip counts in `PullReport`, not as
hard errors. Hard errors are invalid source URL, `nix-cache-info` `StoreDir`
mismatch, PathInfo persistence failure after acceptance, and export failure
after persistence succeeds.

HTTP-specific skip-reason mapping to existing `PullReport` fields:
- Narinfo 403/404 → `skipped_missing_nar_count`
- Other non-success non-redirect narinfo HTTP statuses (including 400/401/410/5xx) → `skipped_parse_error_count`
- Narinfo parse failure → `skipped_parse_error_count`
- Requested-path / narinfo `StorePath` mismatch → `skipped_parse_error_count`
- Network/timeout error while fetching narinfo or NAR → `skipped_parse_error_count`
- Untrusted signature → `skipped_untrusted_count`
- NAR non-success non-redirect HTTP statuses (including 401/403/404/410/5xx) → `skipped_missing_nar_count`
- Redirect rejection for narinfo or NAR → `skipped_parse_error_count`
- NAR hash mismatch or decompression failure during ingestion → `skipped_hash_mismatch_count`
- Narinfo store-directory prefix mismatch → `skipped_store_dir_mismatch_count`
- Absolute URL in narinfo → `skipped_parse_error_count`

`skipped_untrusted_count` already exists in `PullReport` (used by directory
pull). No new fields are needed.

**Rationale:** A remote cache may have partial coverage. Skipping missing
paths and reporting counts lets the operator see what succeeded vs. failed
without aborting the entire pull, while persistence/export failures remain
fatal because the import has already crossed the trust boundary.

### 10. Cache base URL normalization keeps path prefixes stable

**Choice:** Normalize the parsed cache base URL so `--from https://host/cache`,
`--from https://host/cache/`, and forms with ignored query/fragment
components all resolve `nix-cache-info`, narinfo, and relative nar paths under
`/cache/` rather than dropping the last segment. Strip query and fragment
components before any `Url::join(...)` call. Reject cache URLs with userinfo
instead of attempting to normalize embedded credentials.

**Rationale:** Path-prefixed caches are common behind nginx, S3 website roots,
and similar static hosting. Without explicit normalization, `Url::join` can
silently fetch the wrong objects.

### 11. Persist via pathinfo_service.put() + export to disk

**Choice:** Same as directory pull — `put()` the constructed PathInfo, then
`export_castore_to_disk()`. No `persist_and_export_signed_output()`.
Persistence/export failures abort the whole call immediately.

**Rationale:** Consistency with directory pull. Imported paths have narinfo
signatures as trust anchors, not build provenance.

### 12. Decompression dispatch: inline, not extracted

**Choice:** Phase 1 inlines the compression-format match in
`import_paths_from_http_cache()`, duplicating the pattern from
`NixHTTPPathInfoService::get()`. Extraction into a shared helper is a
follow-up.

**Rationale:** The match is ~15 lines. Extracting it requires touching
vendored `snix-store` code and adds coupling. Inline duplication is
acceptable for Phase 1; a shared helper becomes worthwhile if a third
caller appears.

### 13. Reqwest client configuration

**Choice:**
- Connect timeout: 30 seconds
- Read timeout: 5 minutes (NARs can be large)
- Redirect policy: none (no redirects followed)
- User-agent: `crunch/<version>`
- No insecure mode, custom CA, certificate-bypass, or TLS-override knobs in
  Phase 1; the client uses reqwest's normal transport validation only
- No maximum response size guard in Phase 1 (NARs are hash-verified,
  so a malicious oversize response is caught at ingestion or wastes
  bandwidth but does not corrupt state)

**Rationale:** Matches typical HTTP client defaults. The timeouts prevent
hangs against misbehaving servers. Disabling redirects entirely closes the
redirect-based SSRF gap noted in the spec.

## Verification Strategy

Warning observability uses the existing warning surface: tests capture emitted
`tracing::warn!` output or stderr warnings and assert the relevant warning
substring before checking the skip/fail counters.


| Spec scenario | Test type | Method |
|---|---|---|
| Pull single path from HTTP cache | Unit | Push to dir, serve with `TcpListener` + manual HTTP response, pull via `import_paths_from_http_cache` |
| Pull skips already-present local PathInfo after mandatory preflight | Unit | Seed local PathInfo, request same digest, assert `/nix-cache-info` is still fetched while narinfo/NAR requests stay at zero |
| Pull skips 404/403 narinfo paths | Unit | HTTP server returns 404 and 403 for narinfo requests and verify `skipped_missing_nar_count` |
| Pull maps other non-success narinfo statuses to parse-error count | Unit | HTTP server returns representative 400/401/410/5xx narinfo statuses and verify `skipped_parse_error_count` |
| Pull handles compressed NAR | Unit | Push to dir, xz-compress the NAR, update narinfo `Compression`/`FileHash`/`FileSize`, serve, pull |
| Pull handles gzip/bzip2/zstd compressed NARs | Unit | Push to dir, serve gzip-, bzip2-, and zstd-compressed NARs, and verify each import succeeds through the normal ingestion path |
| Pull accepts uncompressed NAR input | Unit | Serve narinfo with absent or `none` `Compression` and verify the import succeeds without decompression |
| Pull rejects unknown compression labels | Unit | Serve narinfo with an unknown `Compression` label and verify `skipped_parse_error_count` |
| Pull rejects untrusted signatures | Unit | Serve narinfo signed with an unknown key, verify `skipped_untrusted_count` |
| Pull rejects unsigned narinfo when trust-unsigned is false | Unit | Serve unsigned narinfo, leave `trust_unsigned = false`, and verify `skipped_untrusted_count` |
| Pull accepts unsigned narinfo when trust-unsigned is enabled | Unit | Serve unsigned narinfo, set `trust_unsigned = true`, and verify the import succeeds |
| Pull accepts unknown-key narinfo when trust-unsigned is enabled | Unit | Serve narinfo signed by an unknown key, set `trust_unsigned = true`, and verify the import succeeds |
| Pull rejects absolute/root-relative/escaping narinfo URL values | Unit | Serve narinfo `URL` values that are absolute, root-relative, `//` authority-changing, or path-escaping and verify `skipped_parse_error_count` |
| Narinfo redirect rejection maps to parse-error count | Unit | Serve a narinfo request with `302 Location: ...`, verify the request is not followed, and assert `skipped_parse_error_count` |
| NAR redirect rejection maps to parse-error count | Unit | Serve a relative NAR path with `302 Location: ...`, verify the request is not followed, and assert `skipped_parse_error_count` |
| Pull rejects malformed narinfo text | Unit | Serve malformed narinfo text and verify `skipped_parse_error_count` before later validations run |
| Pull rejects narinfo store-path mismatch | Unit | Serve narinfo whose `StorePath` differs from the requested path, verify `skipped_parse_error_count` before NAR download |
| Pull rejects narinfo store-path prefix mismatch | Unit | Serve matching `nix-cache-info` plus a narinfo whose `StorePath` uses a different store prefix and verify `skipped_store_dir_mismatch_count` before NAR download |
| Pull parses references with the local store prefix | Unit | Use a non-default local store prefix, import a narinfo with valid reference basenames, and verify the imported `PathInfo` retains the references unchanged under that local prefix |
| Pull rejects malformed or mismatched references | Unit | Serve narinfo whose `References` are malformed or absolute paths using the local or a different store prefix and verify `skipped_parse_error_count` before persistence |
| Pull does not recurse into missing references | Unit | Serve one requested narinfo whose references mention absent digests, assert only the requested digest was fetched |
| Matching StoreDir proceeds normally | Unit | Serve `nix-cache-info` with the local `StoreDir` and verify the requested narinfo is fetched and imported |
| Narinfo transport failure continues across remaining paths | Unit | Request two paths, drop the narinfo response for one digest, capture the warning, and verify `skipped_parse_error_count` plus successful import of the other path |
| NAR non-success non-redirect statuses map to missing-nar count | Unit | Serve a valid narinfo whose NAR URL returns representative 401/403/404/410/5xx statuses and verify `skipped_missing_nar_count` |
| NAR transport failure continues across remaining paths | Unit | Request two paths, drop the NAR response for one path, and verify `skipped_parse_error_count` plus successful import of the other path |
| NAR transport failure maps to parse-error count | Unit | Server drops connection during the NAR response and verify `skipped_parse_error_count` |
| Decompression failure or NarHash mismatch maps to hash-mismatch count | Unit | Serve corrupted NAR bytes or a mismatched `NarHash` and verify `skipped_hash_mismatch_count` |
| StoreDir mismatch | Unit | Serve `nix-cache-info` with wrong StoreDir and assert no narinfo request happens before the hard error |
| Missing nix-cache-info | Unit | Server returns 404 for `/nix-cache-info`, capture the warning, and verify pull proceeds |
| Malformed nix-cache-info | Unit | Server returns `/nix-cache-info` without parseable `StoreDir`, capture the warning, and verify pull proceeds |
| Missing nix-cache-info still rejects narinfo prefix mismatch | Unit | Return 404 for `/nix-cache-info`, capture the warning, then serve a narinfo whose `StorePath` uses the wrong store prefix and verify `skipped_store_dir_mismatch_count` before NAR download |
| Malformed nix-cache-info still rejects narinfo prefix mismatch | Unit | Serve malformed `/nix-cache-info`, capture the warning, then serve a narinfo whose `StorePath` uses the wrong store prefix and verify `skipped_store_dir_mismatch_count` before NAR download |
| nix-cache-info request failure | Unit | `/nix-cache-info` times out, refuses, or returns representative non-success non-redirect statuses such as 401/410/5xx, capture the warning, and verify pull then continues to narinfo fetch |
| nix-cache-info request failure still rejects narinfo prefix mismatch | Unit | Make `/nix-cache-info` fail, capture the warning, then serve a narinfo whose `StorePath` uses the wrong store prefix and verify `skipped_store_dir_mismatch_count` |
| nix-cache-info redirect rejection warns and continues | Unit | `/nix-cache-info` returns `302 Location: ...`, capture the warning, and verify pull then continues to narinfo fetch |
| Path-prefixed cache URL resolution stays under the prefix | Unit | Use `https://host/cache`, `https://host/cache/`, and query/fragment variants and verify `nix-cache-info`, narinfo, and relative NAR fetches stay under `/cache/` |
| Library rejects cache URL userinfo | Unit | Call `import_paths_from_http_cache(...)` with `https://user@host/cache` and verify it errors before any HTTP request |
| PathInfo persistence failure is fatal | Unit | Inject a failing `PathInfoService::put()` after valid narinfo/NAR validation and verify the whole call errors instead of reporting a skip |
| Export failure after persistence is fatal | Unit | Persist the imported `PathInfo`, force export to an invalid output root, and verify the whole call errors instead of reporting a skip |
| Missing explicit path selectors over HTTP | Integration | CLI returns error for `store pull --from https://...` with no paths and no `--all`, using an observing fixture to prove no HTTP request is issued |
| Unsupported URL schemes are rejected before dispatch | Integration | `store pull --from file://...` fails with a clear error before local-path dispatch or any HTTP request |
| HTTP URL userinfo is rejected before dispatch | Integration | `store pull --from https://user@cache.example.com ...` fails with a clear error before any HTTP request |
| --all rejection over HTTP | Integration | CLI returns error for `--all --from https://...` |
| CLI URL dispatch succeeds | Integration | Seed a signed cache dir, serve it over a real local HTTP fixture, run `crunch store pull --from http://127.0.0.1:PORT <logical-path>`, and verify the imported output exists on disk |
| Fresh-store HTTP round-trip uses the source verifying key | Integration | Push a cache to disk, serve it over local HTTP, pull into a fresh store/state dir with the source verifying key, and verify the output exists on disk |
| Push → HTTP-serve → pull round-trip | Integration | Build, push, reopen signed PathInfo from the first store state, serve `nix-cache-info` + narinfo + NAR bytes from disk over a real local HTTP fixture, pull from URL into a fresh state dir with the first store's verifying key, and verify PathInfo + on-disk output |

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
by redirect-disablement plus relative-URL-only narinfo resolution.
