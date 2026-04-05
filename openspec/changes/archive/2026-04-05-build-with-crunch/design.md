# Build with Crunch — Design

## Context

crunch has a working eval→build pipeline: Nickel eval produces derivation
JSON, the orchestrator resolves dependencies, bwrap sandboxes run builders,
outputs get NAR-hashed and persisted as PathInfo. But the only confirmed
working build is `hello.ncl` (echo to $out via /bin/sh).

The `/nix/store` on this machine is read-only, so all builds use
`--store /tmp/crunch-store`. Source inputs still read from `/nix/store/`.

The seed (`examples/seed.ncl`) contains: bash, coreutils, gcc, gnumake,
binutils — all as pre-existing nix store paths. Closure resolution
(`nix-store -qR`) pulls transitive deps into the sandbox.

## Goals / Non-Goals

**Goals:**
- Verify every major crunch subsystem via concrete builds
- Discover and fix failures at each tier before moving to the next
- Leave behind working examples that double as regression tests

**Non-Goals:**
- Performance benchmarking
- Cross-compilation or non-x86_64 targets
- Daemon mode or multi-user store access
- Parallel build scheduling (sequential is fine)

## Tier Design

### Tier 1 — Smoke (no seed, no network)

Exercises: eval, sandbox invocation, CA output path computation, PathInfo
persistence, build caching.

| Test | What it validates |
|------|-------------------|
| `echo > $out` via `/bin/sh` | Core pipeline (already works) |
| Multi-line script, exit code propagation | Error path |
| Rebuild skip (run twice, second is instant) | Build caching |

Builder is `/bin/sh` — the only thing guaranteed available without seed.

### Tier 2 — Seed toolchain (bash + coreutils + gcc)

Exercises: `crunch bootstrap`, closure resolution, multi-input sandbox
mounts, `inputs` array handling.

| Test | What it validates |
|------|-------------------|
| `hello-world.ncl` — compile C with gcc | Seed inputs mount correctly |
| Run the built binary | Output directory structure works |
| Build with gnumake | Make + gcc integration |

Requires: `crunch bootstrap -o examples/seed.ncl` to generate/refresh
the seed. The seed paths must exist in `/nix/store/`.

### Tier 3 — Fetchers (network required)

Exercises: `builtin:fetchurl` bypass, FOD hash verification, tarball
extraction, git clone.

| Test | What it validates |
|------|-------------------|
| `fetch-file.ncl` — single file by URL | fetchurl + hash check |
| `fetch-tarball.ncl` — tarball download + extract | Tarball pipeline |
| `fetch-git.ncl` — git clone at rev | Git fetcher |
| Wrong hash → build failure | Hash mismatch error path |

These bypass the sandbox (builtin:fetchurl). Network access required.

### Tier 4 — Composition (multi-output, deps, mkDerivation)

Exercises: output selection (`crunch.select`), inter-derivation deps,
mkDerivation phase system, overrideAttrs.

| Test | What it validates |
|------|-------------------|
| `multi-output.ncl` — $out + $dev | Multi-output handling |
| `package-set.ncl` — libfoo + app | Dependency ordering |
| `mk-hello.ncl` — mkDerivation | Phase system (build/install) |
| `override.ncl` — overrideAttrs | Nickel merge + rebuild |

### Tier 5 — Real package

Build something real from an upstream tarball. Candidate: jq (small C
project, autoconf, few deps) or a tiny Rust crate (if rustc is in seed).

| Test | What it validates |
|------|-------------------|
| fetchTarball → configure → make → install | End-to-end real build |
| Built binary runs and produces correct output | Functional binary |

## Decisions

### 1. Use `--store /tmp/crunch-store` everywhere

**Choice:** All builds write to `/tmp/crunch-store`.
**Rationale:** `/nix/store` is read-only on this machine. The `--store`
flag controls only where outputs land; source inputs always read from
`/nix/store/`.

### 2. Run tiers sequentially, stop on first failure

**Choice:** Don't skip ahead. Tier N failures indicate problems that
would cascade into Tier N+1.
**Rationale:** A broken sandbox won't help debugging a broken mkDerivation.

### 3. Clear the store between tiers

**Choice:** `rm -rf /tmp/crunch-store/*` between tiers to test from a
clean slate. Optionally re-run tier 1 without clearing to verify caching.
**Rationale:** Leftover PathInfo from a previous run can mask bugs.

## Risks / Trade-offs

**[Seed staleness]** → Seed paths reference specific nix store paths.
If the system's nixpkgs moves, paths disappear. Mitigation: `crunch
bootstrap` regenerates from whatever's installed. Pin as GC roots.

**[Network flakiness]** → Tier 3 fetchers hit the network. Mitigation:
Use small files, stable URLs (GitHub raw files, tagged releases).

**[bwrap permissions]** → Unprivileged bwrap needs user namespaces.
If `kernel.unprivileged_userns_clone` is 0, sandbox fails. Mitigation:
check upfront and skip tier 2+ if unavailable.
