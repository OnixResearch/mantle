## Phase 1: Fetch execution in crunch-build

- [x] Add HTTP/compression deps to `crunch-build/Cargo.toml`: ureq (rustls), flate2 (rust_backend), tar, lzma-rs, bzip2-rs, ruzstd
- [x] Create `crates/crunch-build/src/fetcher.rs` with local `Fetch` enum (URL, Tarball, NAR, Executable, Git) adapted from snix-redox
- [x] Implement `parse_fetch_from_derivation()`: read `env.url`, `env.unpack`, `env.type`, `env.rev`, `env.executable` from `nix_compat::Derivation` → `Fetch`
- [x] Implement `fetch_to_store()`: dispatch on `Fetch` variant → download + optional extract + write to output path (sync, ureq-based)
- [x] Implement `decompress_reader()`: select decompressor by URL suffix (gz/xz/bz2/zst/raw), gzip magic fallback
- [x] Implement `extract_tar()`: tar extraction with top-level prefix stripping (from snix-redox)
- [x] Implement `fetch_git()`: git clone --bare + checkout + strip .git/ (from snix-redox)
- [x] Implement `verify_fetch_hash()`: flat hash (raw bytes) and recursive hash (NAR) verification against declared FOD hash
- [x] Unit tests for `parse_fetch_from_derivation()` (all Fetch variants + missing fields + non-fetcher derivation)
- [x] Unit tests for `extract_tar()` (empty tar, single file, prefix stripping, symlinks, hard links)
- [x] Unit test for `decompress_reader()` (gz, xz, bz2, zst, raw, unknown-with-gzip-magic)

## Phase 2: Orchestrator integration

- [x] Add `is_builtin_fetcher()` check in `build_derivation_inner()`: if `derivation.builder.starts_with("builtin:")`, branch to fetcher path
- [x] Validate fetcher is FOD: reject `builder = "builtin:fetchurl"` without `fixed_output` (return `Error::FetcherNotFod`)
- [x] Wrap `fetch_to_store()` in `tokio::task::spawn_blocking` (ureq is sync)
- [x] After fetch: run existing post-build pipeline (NAR hash, reference scan, PathInfo persist) — same code path as regular builds
- [x] Handle CA derivation fetchers: FODs compute output path from declared hash regardless of addressing_mode (existing behavior, verify it works)
- [x] Integration test: fetchurl a small file from a local HTTP server, verify store path exists
- [x] Integration test: fetchTarball with .tar.gz, verify extracted contents and prefix stripping

## Phase 3: Nickel stdlib fetch helpers

- [x] Create `lib/fetch.ncl` with `fetchurl`, `fetchTarball`, `fetchGit` functions
- [x] `fetchurl`: URL + hash → FOD record with `builder = "builtin:fetchurl"`, `mode = 'flat`, env.url
- [x] `fetchTarball`: URL + hash → FOD record with `mode = 'recursive`, `env.unpack = "1"`
- [x] `fetchGit`: URL + rev + hash → FOD record with `mode = 'recursive`, `env.type = "git"`, `env.rev`
- [x] URL validator contract: non-empty string, starts with `http://` or `https://` or `file://`
- [ ] Hash validator contract: matches SRI format — deferred (Nickel recursive record scoping prevents inline contract use; validation done in Rust glue)
- [x] Name derivation: default to URL basename, override with explicit `name` parameter
- [x] Re-export from `lib/lib.ncl`: `fetchurl`, `fetchTarball`, `fetchGit`
- [x] Unit tests: valid fetchurl/fetchTarball/fetchGit produce correct derivation JSON
- [x] Unit tests: contract violations for empty URL, invalid hash, missing rev (fetchGit)

## Phase 4: Hash mismatch reporting + auto-fix

- [ ] On FOD hash mismatch in fetcher, compute correct hash in SRI format
- [ ] Format error message: expected hash, actual hash, source file + suggested update
- [ ] Add `--fix` flag to `crunch build` CLI
- [ ] Implement `auto_fix_hash()`: read .ncl source, find old SRI/hex hash string, replace with correct SRI, write back
- [ ] Guard: only fix if old hash string appears exactly once in the file (warn + skip if ambiguous)
- [ ] Integration test: fetchurl with wrong hash, verify error message contains correct hash
- [ ] Integration test: `--fix` rewrites hash in .ncl file and build succeeds on retry

## Phase 5: End-to-end examples

- [ ] `examples/fetch-file.ncl`: fetchurl a single file (e.g., a small known-hash text file)
- [ ] `examples/fetch-tarball.ncl`: fetchTarball a GitHub release → use contents in a build
- [ ] `examples/fetch-git.ncl`: fetchGit a repo → build something from the checkout
