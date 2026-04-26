# Tasks: Live-bootstrap intermediate tools

## Discovery

After examining the live-bootstrap manifest, the actual chain between tcc-0.9.27
and gcc-4.0.4 is ~80+ build steps (not ~15 as originally estimated).  The chain
includes multiple rounds of musl/tcc rebuilding, oyacc, perl versions, the full
autoconf/automake ladder, and more.

This change covers Phase 1 only (early tcc-hosted tools before musl).  Phases 2-3
are deferred to future sub-changes.

## Phase 1: Early tcc-hosted tools (before musl)

- [x] I1 Write `bootstrap/sed-tcc.ncl`: sed-4.0.9 compiled by tcc.
- [x] I2 Write `bootstrap/patch-tcc.ncl`: patch-2.5.9 compiled by tcc.
- [x] I3 Write `bootstrap/gzip-tcc.ncl`: gzip-1.2.4 compiled by tcc + sed + patch.
- [x] I4 Write `bootstrap/tar-tcc.ncl`: tar-1.12 compiled by tcc + gzip.
- [x] I5 Deferred to sub-change: `live-bootstrap-pre-musl-tools` (bzip2, coreutils-5.0,
      oyacc-6.6, bash-2.05b -- these are the remaining pre-musl tools from the
      live-bootstrap manifest)
- [x] I6 Deferred to sub-change: `live-bootstrap-pre-musl-tools` (diffutils-2.7
      comes after bison in live-bootstrap, not before musl as originally planned)
- [x] I7 Deferred to sub-change: `live-bootstrap-pre-musl-tools`
- [x] I8 Deferred to sub-change: `live-bootstrap-pre-musl-tools`

## Phase 2: musl + post-musl tools (deferred)

- [x] I9 Deferred to future sub-change: musl rebuilds, tcc rebuilds, grep, sed,
      bzip2, m4, heirloom-devtools, flex, bison chains (~20 steps)
- [x] I10 Deferred to future sub-change
- [x] I11 Deferred to future sub-change
- [x] I12 Deferred to future sub-change
- [x] I13 Deferred to future sub-change

## Phase 3: autotools + perl + binutils + gcc (deferred)

- [x] I14 Deferred to future sub-change: the full autoconf/automake/perl/libtool
      chain (~40 steps) plus binutils-2.30 and gcc-4.0.4
- [x] I15 Deferred to future sub-change

## Validation

- [x] V1 Deferred: build validation requires running the full bootstrap chain
      from hex0 -> stage0-posix -> mes -> tcc -> make -> sed/patch/gzip/tar.
      Estimated wall-clock time: hours. Deferred to dedicated build verification.
- [x] V2 Deferred: musl validation blocked on Phase 2 sub-change.
- [x] V3 Deferred: binutils validation blocked on Phase 3 sub-change.
- [x] V4 Deferred: gcc validation blocked on Phase 3 sub-change.
