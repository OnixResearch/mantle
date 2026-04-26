# Tasks: Live-bootstrap intermediate tools

## Phase 1: Early tcc-hosted tools

- [ ] I1 Write `bootstrap/sed-tcc.ncl`: sed-4.0.9 compiled by tcc. Adapt from
      live-bootstrap `steps/sed-4.0.9/`.
- [ ] I2 Write `bootstrap/patch-tcc.ncl`: patch-2.5.9 compiled by tcc. Adapt
      from live-bootstrap `steps/patch-2.5.9/`.
- [ ] I3 Write `bootstrap/gzip-tcc.ncl`: gzip-1.2.4 compiled by tcc.
- [ ] I4 Write `bootstrap/tar-tcc.ncl`: tar-1.12 compiled by tcc.
- [ ] I5 Write `bootstrap/gawk-tcc.ncl`: gawk-3.0.4 compiled by tcc.
- [ ] I6 Write `bootstrap/diffutils-tcc.ncl`: diffutils-2.7 compiled by tcc.
- [ ] I7 Write `bootstrap/bash-tcc.ncl`: bash-2.05b compiled by tcc.
- [ ] I8 Write `bootstrap/coreutils-tcc.ncl`: coreutils-5.0 compiled by tcc.

## Phase 2: musl libc and musl-linked tools

- [ ] I9 Write `bootstrap/musl-tcc.ncl`: musl-1.1.24 compiled by tcc.
      First real C library in the chain.
- [ ] I10 Write `bootstrap/m4-tcc.ncl`: m4-1.4.7 compiled by tcc + musl.
- [ ] I11 Write `bootstrap/flex-tcc.ncl`: flex-2.5.11 compiled by tcc + musl.
- [ ] I12 Write `bootstrap/bison-tcc.ncl`: bison-2.3 compiled by tcc + musl.
- [ ] I13 Write `bootstrap/grep-tcc.ncl`: grep-2.4 compiled by tcc + musl.

## Phase 3: binutils and gcc-4.0.4

- [ ] I14 Replace `bootstrap/binutils-tcc.ncl` placeholder with functional
      derivation: binutils-2.30 compiled by tcc + musl + all Phase 1-2 tools.
- [ ] I15 Replace `bootstrap/gcc-4.0.ncl` placeholder with functional derivation:
      gcc-4.0.4 compiled by tcc + binutils-2.30 + musl + all Phase 1-2 tools.

## Validation

- [ ] V1 Each Phase 1 derivation builds with `crunch build`.
- [ ] V2 musl-1.1.24 builds and tcc can link against it.
- [ ] V3 binutils-2.30 builds and produces working `as`, `ld`, `ar`.
- [ ] V4 gcc-4.0.4 builds and can compile a C program.
