# Nix-Free Bootstrap — Design

## Context

crunch's architecture already supports Nix-free operation. The
`bootstrap-no-nix.ncl` proof showed:

- `fetchTarball` downloads and unpacks without a sandbox (builtin fetcher)
- Static binaries need no closure resolution (`nix-store -qR` returns empty)
- Fetched outputs become `input_derivations`, not `input_sources` — the
  orchestrator looks them up in `--store`, not `/nix/store`
- busybox-static (mounted as `/bin/sh` + `/bin/busybox`) provides shell
  and basic utilities in the sandbox

The remaining Nix dependencies are in two code paths, both removable.

## Goals / Non-Goals

**Goals:**
- `crunch bootstrap --fetch` produces a working seed from fetched tarballs
- A full C project builds with zero Nix binaries on PATH
- The bootstrap chain is reproducible (pinned hashes for all fetches)
- Existing `--from-nix` mode continues to work

**Non-Goals:**
- Building rustc from source (fetch a static rustc instead)
- Cross-compilation support
- Replacing nixpkgs with a Nickel package set (separate project)
- Bit-for-bit Nix store path compatibility

## Decisions

### 1. Static musl-gcc as the root of trust

**Choice:** Fetch a pre-built static musl-gcc from musl.cc as the
bootstrap compiler.

**Rationale:** musl.cc provides a single ~89MB tarball containing a
complete static C toolchain (gcc, g++, ar, ld, as, strip). All binaries
are statically linked against musl, so they run anywhere with zero
dependencies. The tarball is reproducible and versioned.

**Alternative rejected:** stage0-posix (hex0 → M2 → mes → tcc → gcc).
Full auditable bootstrap from source. Correct long-term but requires
months of packaging work. Use the pre-built gcc now, replace the root
of trust later.

**Alternative rejected:** cosmopolitan libc / cosmocc. Interesting but
adds a non-standard ABI dependency. musl is standard and well-understood.

### 2. Keep busybox-static as sandbox shell

**Choice:** Continue using busybox-static as `SNIX_BUILD_SANDBOX_SHELL`.
Mount it at both `/bin/sh` and `/bin/busybox` in the sandbox.

**Rationale:** busybox provides sh + 300 applets in a single static
binary. Good enough for build scripts. The `/bin/busybox` mount (added
in this session) allows `busybox <applet>` invocation and symlink
creation for PATH-based applet access.

**Future:** Once crunch can build bash from source, use that as the
default builder. Keep busybox as the minimal fallback.

### 3. Fetch-based seed.ncl generation

**Choice:** `crunch bootstrap --fetch` downloads tarballs via the
existing ureq fetcher, unpacks them, persists outputs as FODs, and
writes `seed.ncl` referencing the crunch store paths.

**Implementation:**
```
crunch bootstrap --fetch --store /path/to/store -o seed.ncl
```

This runs `fetchTarball` internally (same code path as `builtin:fetchurl`
derivations) for each seed package:

| Package | Source | Hash |
|---------|--------|------|
| musl-gcc | musl.cc/x86_64-linux-musl-native.tgz | sha256-XpcI34... |
| busybox | busybox.net static build (or bundled) | pinned |
| gnumake | fetched tarball + built with musl-gcc | computed |

The generated `seed.ncl` references crunch store paths:
```nickel
{
  gcc = "/nix/store/xxx-musl-gcc",  # logical prefix, physical at --store
  coreutils = "/nix/store/yyy-busybox",
  make = "/nix/store/zzz-gnumake",
}
```

Note: store paths still use the `/nix/store` logical prefix (derivation
path computation requires it). The physical location is controlled by
`--store`.

### 4. Closure resolution: static-aware fast path

**Choice:** Before calling `nix-store -qR`, check if the input is a
crunch-built output (exists in the output store, not a Nix-provided
source). If so, skip closure resolution — crunch-built outputs already
have their dependencies tracked via `input_derivations`.

**Implementation:** In `collect_source_paths()`, only call
`resolve_nix_closure()` for paths that exist in the physical
`/nix/store/` (actual Nix store), not for paths in `--store`.

### 5. Phase 3 build order

Build core tools in dependency order, each using the previous stage's
output:

```
musl-gcc (fetched)
  → make (from source, built with musl-gcc)
  → bash (from source, built with musl-gcc + make)
  → coreutils (from source, or keep busybox)
  → patch, sed, grep, awk, findutils
```

Each builds as a `crunch.Derivation` using `mkDerivation` with the
fetched seed. Each is a FOD or CA derivation — reproducible and cached.

These are Nickel files in a `bootstrap/` directory, not hard-coded in
Rust. crunch builds them like any other derivation.

## Risks / Trade-offs

**[Root of trust]** → The fetched musl-gcc is a pre-built binary from a
third party. We trust musl.cc to not ship a backdoored compiler. The
hash pin ensures the tarball content is fixed, but not that the original
build was honest. Mitigation: long-term, replace with stage0-posix
(auditable from hex).

**[musl vs glibc]** → Programs built with musl-gcc link against musl
libc, not glibc. Most C programs work fine with musl, but some
(especially ones using glibc extensions: `dlopen`, `nss`, `locale`)
may not. Mitigation: the Phase 4 gcc rebuild can target glibc if needed.

**[Tarball availability]** → musl.cc could go offline. Mitigation: mirror
the tarball in crunch's own infrastructure. The hash pin means any mirror
with the right content works.

**[Scope creep]** → Building gcc from source (Phase 4) is a multi-week
project. Mitigation: Phase 4 is optional — Phases 1-3 provide a working
Nix-free build environment for most C projects. Phase 4 is only needed
for self-hosting the compiler.
