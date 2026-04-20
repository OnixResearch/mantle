## Phase 1: Store library — HTTP pull core

- [ ] Add `PullSource` enum (`Directory(PathBuf)`, `Http(Url)`) to `crates/crunch-store/src/pull.rs`
- [ ] Implement `fetch_remote_nix_cache_info(client, url) -> Result<Option<String>>` — fetch `/nix-cache-info`, parse `StoreDir`, return it or `None` on 404
- [ ] Implement `import_paths_from_http_cache(handle, url, paths, options) -> Result<PullReport>`:
  - Validate remote `nix-cache-info` StoreDir (hard error on mismatch, warn-and-proceed on absent)
  - 1. Check local PathInfo existence before fetching narinfo (save network round-trip)
  - 2. Fetch narinfo by digest, verify signatures (reuse `PullOptions.trust_unsigned` + `trusted_public_keys`)
  - Reject absolute URLs in narinfo `URL` field (SSRF mitigation per design Decision 5)
  - Download NAR, decompress inline (bzip2/gzip/xz/zstd match per design Decision 10)
  - Ingest via `ingest_nar_and_hash`, verify NAR hash
  - Persist PathInfo, export to disk
  - Build reqwest `Client` with: 30s connect timeout, 5min read timeout, same-scheme redirect policy (max 10 hops), `crunch/<version>` user-agent (per design Decision 11)
- [ ] Unit test: HTTP pull happy path — push to dir, serve with `TcpListener` on `127.0.0.1:0`, pull via `import_paths_from_http_cache`, verify PathInfo + report
- [ ] Unit test: idempotent re-pull — import once, then re-import same path; verify skipped without narinfo network fetch and `skipped_already_present_count = 1`
- [ ] Unit test: 404 narinfo skip — server returns 404 for narinfo request, verify `skipped_missing_nar_count`
- [ ] Unit test: untrusted signature rejection — serve narinfo signed by unknown key, verify `skipped_untrusted_count`
- [ ] Unit test: NAR hash mismatch — serve narinfo + tampered NAR content, verify `skipped_hash_mismatch_count`
- [ ] Unit test: NAR download failure — narinfo fetched successfully but NAR request returns 500, verify `skipped_missing_nar_count`
- [ ] Unit test: network error / dropped connection — server drops connection mid-NAR-response, verify path is skipped and pull continues for remaining paths
- [ ] Unit test: store-prefix mismatch via nix-cache-info — serve `StoreDir: /wrong/store`, verify hard error returned
- [ ] Unit test: missing nix-cache-info (404) — verify warning logged and pull proceeds successfully
- [ ] Unit test: absolute URL in narinfo `URL` field — verify rejected with `skipped_parse_error_count`
- [ ] Unit test: compressed NAR fetch — xz-compress pushed NAR, update narinfo Compression/FileHash/FileSize, serve, verify import succeeds
- [ ] Unit test: cross-scheme redirect rejection — HTTP server returns 302 with `file://` Location, verify request fails or is blocked by same-scheme policy
- [ ] Unit test: reqwest client config — assert 30s connect timeout, 5min read timeout, same-scheme redirect policy, `crunch/<version>` user-agent header sent to server
- [ ] Unit test: StorePath mismatch — serve narinfo whose `StorePath` differs from the requested store path, verify path is skipped with appropriate report count

## Phase 2: CLI integration

- [ ] Detect `http://` / `https://` prefix in `--from` value in `cmd_store_pull()` and dispatch to `import_paths_from_http_cache`
- [ ] Reject `--all` with HTTP source with a clear error message
- [ ] Parse path selectors into `StorePath` for HTTP digest-based lookup
- [ ] Integration test: build → push to dir → serve over HTTP → `crunch store pull --from http://127.0.0.1:PORT ...` → verify PathInfo matches and output exists on disk
- [ ] Integration test: `--all --from http://...` returns clear error message

## Phase 3: Documentation

- [ ] Update README Binary Cache Sharing section with HTTP pull examples (`crunch store pull --from https://cache.example.com ...`)
