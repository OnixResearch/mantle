# Nix-Free Bootstrap — Tasks

## Phase 1: Fetch-based seed

- [x] Add `--fetch` flag to `crunch bootstrap` CLI (clap) ✅ 10m (started: 2026-04-05T16:43Z → completed: 2026-04-05T16:45Z)
- [x] Implement `bootstrap_fetch()`: download musl-gcc tarball via ureq, unpack, persist as FOD in the crunch store ✅ 10m
- [x] Generate `seed.ncl` referencing the fetched store path (`generate_fetch_seed_ncl` with no GC root comments) ✅ 5m
- [x] Pin the musl-gcc hash in the bootstrap code (sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=) ✅ (in FETCH_SEEDS const)
- [x] Test: `crunch bootstrap --fetch --store /tmp/crunch-store -o seed.ncl` produces valid seed.ncl ✅ 5m (e2e verified)
- [x] Test: a derivation using the fetched seed compiles and runs a C program ✅ 5m (bootstrap-no-nix.ncl with --store /tmp/crunch-fetch-test)

## Phase 2: Eliminate nix-store -qR for crunch-built inputs

- [x] In `collect_source_paths()`: detect whether a source path is a crunch-built output (exists at `--store` prefix) vs a Nix-provided source (exists at `/nix/store/`) ✅ 5m (is_crunch_built() method)
- [x] Skip `resolve_nix_closure()` for crunch-built paths — they have no closure, deps are tracked via `input_derivations` ✅ 5m
- [x] Keep `resolve_nix_closure()` for Nix-provided sources (backward compat with `--from-nix` seeds) ✅ (preserved in resolve_and_ingest_sources)
- [x] Test: build with `--fetch` seed succeeds when `nix-store` is not on PATH ✅ (e2e: bootstrap-no-nix.ncl built successfully)
- [x] Test: build with `--from-nix` seed still works (regression) ✅ (401 unit+integration tests pass)

## Phase 3: Build core tools from source

- [x] Write `bootstrap/make.ncl`: fetch gnumake tarball, build with musl-gcc from seed ✅ 25m
- [x] Test: `crunch build bootstrap/make.ncl` produces a working `make` binary ✅ (GNU Make 4.4.1, static-pie, musl-linked)
- [x] Write `bootstrap/dash.ncl`: build dash 0.5.12 with musl-gcc (pivoted from bash — dash is 28 source files vs bash's 150+, sufficient for build scripts) ✅
- [x] Test: the built dash works (`dash -c` for loops, variable expansion, subshells) ✅ (static-pie ELF, 208KB)
- [~] Write `bootstrap/coreutils.ncl` (or validate busybox is sufficient) — busybox covers all needs, skipped
- [~] Write `bootstrap/sed.ncl`, `bootstrap/grep.ncl`, `bootstrap/awk.ncl` — busybox covers all needs, skipped
- [x] Integration test: multi-file C project (3 .c, 2 .h, Makefile) built with ONLY bootstrap tools ✅ (16 assertions, direct compile + make rebuild both pass)

## Phase 4: Rebuild gcc from source (optional, high-effort)

- [x] Write `bootstrap/binutils.ncl`: fetch + build binutils 2.42 with musl-gcc ✅ (15 tools: as, ld, ar, nm, objcopy, etc.)
- [x] Write `bootstrap/musl.ncl`: fetch + build musl 1.2.5 libc from source ✅ (libc.a, libc.so, ld-musl, 91 headers, 9 static libs)
- [x] Write `bootstrap/gcc.ncl`: build GCC 13.3.0 C-only from source with GMP/MPFR/MPC in-tree ✅
- [~] Test: the crunch-built gcc can compile itself (3-stage bootstrap) — future work
- [~] Remove the fetched musl-gcc from the dependency tree — future work (need to wire gcc+binutils+musl as the compiler for subsequent builds)

## Phase 5: Self-host crunch (stretch)

- [ ] Fetch a static Rust toolchain (rustup target x86_64-unknown-linux-musl)
- [ ] Write `bootstrap/crunch.ncl`: build crunch from its own source using the fetched rustc
- [ ] Test: the crunch-built crunch can build `bootstrap-no-nix.ncl`
- [ ] Alternative: build rustc from source using Phase 4 gcc (very ambitious, separate openspec)
