# Nix-Free Bootstrap — Tasks

## Phase 1: Fetch-based seed

- [ ] Add `--fetch` flag to `crunch bootstrap` CLI (clap)
- [ ] Implement `bootstrap_fetch()`: download musl-gcc tarball via ureq, unpack, persist as FOD in the crunch store
- [ ] Generate `seed.ncl` referencing the fetched store path (reuse `generate_seed_ncl` with crunch paths instead of nix paths)
- [ ] Pin the musl-gcc hash in the bootstrap code (sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=)
- [ ] Test: `crunch bootstrap --fetch --store /tmp/crunch-store -o seed.ncl` produces valid seed.ncl
- [ ] Test: a derivation using the fetched seed compiles and runs a C program (existing `bootstrap-no-nix.ncl` pattern but using the generated seed)

## Phase 2: Eliminate nix-store -qR for crunch-built inputs

- [ ] In `collect_source_paths()`: detect whether a source path is a crunch-built output (exists at `--store` prefix) vs a Nix-provided source (exists at `/nix/store/`)
- [ ] Skip `resolve_nix_closure()` for crunch-built paths — they have no closure, deps are tracked via `input_derivations`
- [ ] Keep `resolve_nix_closure()` for Nix-provided sources (backward compat with `--from-nix` seeds)
- [ ] Test: build with `--fetch` seed succeeds when `nix-store` is not on PATH
- [ ] Test: build with `--from-nix` seed still works (regression)

## Phase 3: Build core tools from source

- [ ] Write `bootstrap/make.ncl`: fetch gnumake tarball, build with musl-gcc from seed
- [ ] Test: `crunch build bootstrap/make.ncl` produces a working `make` binary
- [ ] Write `bootstrap/bash.ncl`: fetch bash tarball, build with musl-gcc + make
- [ ] Test: the built bash works as a builder in subsequent derivations
- [ ] Write `bootstrap/coreutils.ncl` (or validate busybox is sufficient): fetch + build with musl-gcc
- [ ] Write `bootstrap/sed.ncl`, `bootstrap/grep.ncl`, `bootstrap/awk.ncl`: minimal text processing tools
- [ ] Integration test: build a multi-file C project using ONLY Phase 3 tools (no nix paths in the dependency tree)

## Phase 4: Rebuild gcc from source (optional, high-effort)

- [ ] Write `bootstrap/binutils.ncl`: fetch + build binutils with musl-gcc
- [ ] Write `bootstrap/musl.ncl`: fetch + build musl libc from source
- [ ] Write `bootstrap/gcc.ncl`: fetch gcc source, build with Phase 3 tools + musl-gcc
  - Cross-compile gcc targeting musl (avoids glibc dependency)
  - This is the hardest step — gcc's build system is complex
- [ ] Test: the crunch-built gcc can compile itself (3-stage bootstrap)
- [ ] Remove the fetched musl-gcc from the dependency tree — everything traces to source

## Phase 5: Self-host crunch (stretch)

- [ ] Fetch a static Rust toolchain (rustup target x86_64-unknown-linux-musl)
- [ ] Write `bootstrap/crunch.ncl`: build crunch from its own source using the fetched rustc
- [ ] Test: the crunch-built crunch can build `bootstrap-no-nix.ncl`
- [ ] Alternative: build rustc from source using Phase 4 gcc (very ambitious, separate openspec)
