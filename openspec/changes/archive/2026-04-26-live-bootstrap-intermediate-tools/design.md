# Design: Live-bootstrap intermediate tools

## Overview

Each tool derivation follows a consistent pattern adapted from the live-bootstrap
project's step files.  All derivations are Nickel files under `bootstrap/` that
produce a single output directory containing the built tool.

## Derivation pattern

Each `.ncl` file:

1. Imports its predecessor tools from the prior chain link
2. Fetches the source tarball via `crunch.fetchTarball` with a pinned SRI hash
3. Builds in a sandbox with only the prior chain's outputs mounted
4. Installs to `$out/bin/` (and `$out/lib/`, `$out/include/` where applicable)

The build scripts are POSIX shell adapted from live-bootstrap's `pass1.kaem`
or `pass1.sh` files.  Early tools (sed through coreutils) use kaem or the
prior dash/bash; later tools (m4 through binutils) can use bash once available.

## Dependency graph

```
tcc-0.9.27 + make-3.82
  ├── sed-4.0.9
  ├── patch-2.5.9
  ├── gzip-1.2.4
  ├── tar-1.12
  ├── gawk-3.0.4
  ├── diffutils-2.7
  ├── bash-2.05b
  └── coreutils-5.0
       └── musl-1.1.24 (first real libc)
            ├── m4-1.4.7
            ├── flex-2.5.11
            ├── bison-2.3
            └── grep-2.4
                 └── binutils-2.30
                      └── gcc-4.0.4
```

## Source provenance

All source tarballs come from official upstream mirrors (ftpmirror.gnu.org,
ftp.gnu.org, kernel.org).  Each is pinned by SHA-256 SRI hash in the `.ncl` file.
The live-bootstrap project has validated these exact versions build correctly
in the chain.

## Key build constraints

- **No host tools in sandbox**: Each derivation may only use tools from prior
  chain outputs plus the kaem/catm shell from stage0-posix.
- **tcc limitations**: tcc cannot handle complex configure scripts or C99
  features.  Most Phase 1 tools are built by compiling individual `.c` files
  and linking manually, following live-bootstrap's approach.
- **musl as libc boundary**: Before musl-1.1.24, tools link against mes libc
  (limited).  After musl, tools get proper libc support.  This is why m4/flex/bison
  come after musl.
- **binutils needs everything**: binutils-2.30 requires configure + make + many
  shell tools, so it must come last in the intermediate chain.

## Testing approach

Validation focuses on each tool producing a working binary that can be used by
the next stage.  End-to-end validation (gcc-4.0.4 compiling a C program) is the
final acceptance criterion.
