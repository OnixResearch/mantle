## Context

crunch has a working build pipeline (eval → glue → bwrap sandbox → PathInfo)
but no way to download source code. snix-redox solved this already: its
`fetchers.rs` handles URL/tarball/NAR/executable/git fetches with ureq,
hash verification, and tarball extraction (gz/xz/bz2/zst). The upstream
snix-glue `Fetch` enum and `fetchurl_derivation_to_fetch()` parser convert
Nix-style `builtin:fetchurl` derivations into typed fetch descriptions.

crunch doesn't use snix-glue (it replaces the glue layer with crunch-glue).
But the `Fetch` enum and `fetchurl.rs` parser live in snix-glue which depends
on snix-eval — too much baggage. We need the fetch execution without the
Nix evaluator dependency.

## Goals / Non-Goals

**Goals:**
- `crunch.fetchurl`, `crunch.fetchTarball`, `crunch.fetchGit` in Nickel
- Builtin fetcher bypass in the build orchestrator
- Hash verification with actionable mismatch errors
- `--fix` flag for auto-updating hashes in `.ncl` files
- Reuse snix-redox's proven download/extract/verify code

**Non-Goals:**
- Eval-time fetching (Nix's `builtins.fetchurl` evaluates during eval;
  crunch fetches at build time only)
- Fetch caching beyond the existing FOD cache (if the output path exists,
  skip the fetch — already handled by the build cache)
- Mirror/fallback URLs (v1+)
- `fetchClosure` / binary cache fetching (v1+)

## Decisions

### 1. Fetch code lives in crunch-build, not a vendored crate

**Choice:** Copy the fetch execution logic from snix-redox's `fetchers.rs`
into `crates/crunch-build/src/fetcher.rs`. Do not vendor snix-glue.

**Rationale:** snix-glue depends on snix-eval. The `Fetch` enum and
`fetchurl_derivation_to_fetch()` are small (~80 lines for the parser,
~80 lines for the enum). Copying and adapting is cheaper than managing
the dependency tree. snix-redox's download/extract code is ~300 lines
of sync ureq calls — straightforward to lift.

**Alternative rejected:** Vendoring snix-glue and stripping snix-eval.
Too much surgery for too little code.

### 2. Redefine Fetch enum locally, don't import it

**Choice:** Define a `Fetch` enum in `crunch-build/src/fetcher.rs` that
matches the snix-glue one but is owned by crunch. Parse it from the
derivation's environment directly, not through `fetchurl_derivation_to_fetch()`
(which validates `system == "builtin"` — crunch may not use that convention).

**Rationale:** crunch derivations come through Nickel → crunch-glue, which
produces `nix_compat::Derivation`. The fetch detection is:
`derivation.builder == "builtin:fetchurl"`. The environment contains
`url`, `outputHash`, `outputHashAlgo`, `outputHashMode`, `unpack`,
`executable`, `name`. Parsing these into a local `Fetch` enum is ~30 lines.

### 3. Nickel fetch helpers produce standard FOD derivations

**Choice:** `crunch.fetchurl { url, hash }` expands to a full derivation
record with `builder = "builtin:fetchurl"`, `system = "builtin"`, and
the hash in `fixed_output`. The Rust side recognizes `builder` and
dispatches to the fetcher.

**Rationale:** This matches Nix's approach and means fetch derivations
go through the same pipeline as regular derivations — same caching,
same PathInfo persistence, same store path computation. No special
evaluation path needed.

**Implementation in Nickel:**

```nickel
fetchurl = fun { url, hash, name ? null } =>
  let derived_name = name |> match {
    null => url |> std.string.split "/" |> std.array.last,
    n => n,
  } in
  {
    name = derived_name,
    builder = "builtin:fetchurl",
    system = "builtin",
    args = [],
    outputs = ["out"],
    env = { url = url, },
    inputs = [],
    fixed_output = { hash = hash, algo = 'sha256, mode = 'flat },
    addressing_mode = 'input-addressed,
  } | Derivation
```

`fetchTarball` is similar but with `mode = 'recursive` (NAR hash of
unpacked contents) and `env.unpack = "1"`.

`fetchGit` sets `env.type = "git"`, `env.rev = <rev>`, and
`mode = 'recursive`.

### 4. Detection in orchestrator: check builder string

**Choice:** In `build_derivation_inner()`, before constructing a
`BuildRequest`, check if `derivation.builder == "builtin:fetchurl"`.
If so, call `fetcher::fetch()` instead of `build_service.do_build()`.

**Rationale:** Clean separation. The `BuildService` trait is for sandbox
execution. Fetchers are not sandboxed (they need network access). Mixing
them into `BuildService` would require a `NetworkBuildService` or similar
abstraction — overengineered for v1.

### 5. Auto-fix: scan .ncl source for the old hash, replace

**Choice:** When a FOD hash mismatches, compute the correct hash, then
optionally (`--fix`) find the old hash string in the source `.ncl` file
and replace it. Report file:line even without `--fix`.

**Rationale:** Nickel source files are the single source of truth. The
hash string (SRI format like `sha256-XXXX...`) is unique enough to
search-and-replace safely. This matches the defaults spec requirement.

**Risk:** If the same hash appears in multiple places in the file
(unlikely for SRI strings), the wrong one could be replaced. Mitigation:
only replace inside a `fetchurl`/`fetchTarball`/`fetchGit` call if we
can detect the context, otherwise replace the first occurrence and warn.

### 6. Download dependencies: ureq (sync) in a spawn_blocking

**Choice:** Use `ureq` for HTTP, same as snix-redox. Wrap the sync
download in `tokio::task::spawn_blocking` since the build orchestrator
is async.

**Rationale:** ureq is already proven in snix-redox for the same use
case. An async HTTP client (reqwest) would also work but adds a heavy
dependency. ureq is lighter and we don't need concurrent downloads
within a single fetch.

**Dependencies added to crunch-build:**
- `ureq` (HTTP client, rustls TLS)
- `flate2` (gzip)
- `tar` (tar extraction)
- `lzma-rs` (xz)
- `bzip2-rs` (bz2, pure Rust)
- `ruzstd` (zstd, pure Rust)

All pure-Rust (no C deps) for portability. Matches snix-redox's choices.

### 7. fetchGit: shell out to git binary

**Choice:** Same as snix-redox: find the `git` binary, `git clone --bare`,
`git checkout`, strip `.git/`. Not a library-based implementation.

**Rationale:** libgit2 bindings are heavy and don't support all git
features (partial clone, sparse checkout). The git CLI is available on
every system where crunch runs. snix-redox proved this works.

**Risk:** Requires git in PATH. Mitigation: clear error message if git
is not found, suggest installing it.

## Risks / Trade-offs

**[Network access breaks hermeticity]** Fetchers inherently need network
access, which is why they're FODs — the output is pinned by hash.
Mitigation: fetchers MUST be fixed-output derivations. The orchestrator
MUST reject `builder = "builtin:fetchurl"` without a `fixed_output`.

**[ureq is sync]** Could block the tokio runtime if not wrapped in
`spawn_blocking`. Mitigation: all fetch calls go through
`spawn_blocking`.

**[tar prefix stripping is heuristic]** snix-redox strips the first
path component (GitHub-style tarballs). This may not be correct for all
tarballs. Mitigation: document the behavior, add a Nickel option to
control stripping in the future.
