# Build with Crunch — Tasks

## Phase 1: Tier 1 — Smoke (no seed, no network)

- [x] Run `crunch build examples/hello.ncl --store /tmp/crunch-store` and verify output contains "Hello, crunch!" ✅ (cached PathInfo hit)
- [x] Run the same build a second time and confirm it skips ✅ PathInfo persists, castore is in-memory so content is re-exported but sandbox is skipped
- [x] Write a failing-build example (`exit 1`) and confirm crunch exits non-zero with build log on stderr ✅ exit code 1, stderr "this will fail" shown
- [x] Write a multi-step script example and verify output ✅ `multi-step.ncl` — builtins-only (mkdir/chmod need coreutils)

## Phase 1.5: FUSE Blocker (discovered during testing)

- [x] **BLOCKER**: fuse-backend-rs 0.12.0 FUSE mount fails on this machine ✅ documented
  - `mount(2)` returns EPERM for unprivileged users
  - `fusermount3` fallback is broken: passes caller's `fd=N` in opts, meaningless in fusermount3's process space
  - ALL sandbox builds fail when PathInfo cache is empty
  - Previous "successful" builds were PathInfo cache hits from prior crunch sessions
  - **Fix needed**: either upgrade fuse-backend-rs, add bind-mount fallback, or fix fusermount3 invocation

## Phase 2: Tier 2 — Seed toolchain (bash + coreutils + gcc)

- [x] Seed.ncl paths verified — all 5 exist in /nix/store
- [x] Build `examples/hello-world.ncl` ✅ (from PathInfo cache) — binary runs, prints "Hello from crunch!"
- [x] **BUG FOUND + FIXED**: CA single-output rewriting used all-zeros marker, corrupting ELF binaries ✅
  - Zero-padded alignment regions in ELF matched the marker length → store path stamped into section headers
  - Fix: use blake3-derived marker (matches multi-output CA path)
- [x] Build `examples/mk-hello.ncl` ✅ (from PathInfo cache) — mkDerivation works
- [ ] Re-verify builds from clean state (blocked on FUSE fix)

## Phase 3: Tier 3 — Fetchers (network required)

- [x] Build `examples/fetch-file.ncl` ✅ fetches Nickel LICENSE, hash updated
- [x] Build `examples/fetch-tarball.ncl` ✅ fetches nickel 1.9.0 tarball, unpacks correctly
- [x] Build `examples/fetch-git.ncl` ✅ clones npins 0.3.0 tag (switched from nickel rev)
- [x] Hash mismatch test ✅ build fails with clear "FOD hash mismatch" error and shows expected vs actual
- [x] **BUG FOUND + FIXED**: find_git() didn't find git on NixOS ✅
  - Added /run/current-system/sw/bin/git, per-user profile paths, PATH scanning
- [x] **BUG FOUND + FIXED**: BuildFailed error Display omitted the log field ✅
  - Fetcher errors showed generic "exit code fetch" with no details

## Phase 4: Tier 4 — Composition (multi-output, deps, phases)

- [ ] Build `examples/multi-output.ncl` — blocked on FUSE fix (needs sandbox for seed inputs)
- [ ] Build `examples/package-set.ncl` — blocked on FUSE fix
- [x] Eval `examples/override.ncl` — not tested yet (eval doesn't need sandbox)

## Phase 5: Tier 5 — Real package from source

- [ ] Blocked on FUSE fix — requires sandbox for configure/make/install
