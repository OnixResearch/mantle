# Implement binutils-tcc live-bootstrap chain

## Why

Parent change `live-bootstrap-source-chain` inventoried the missing
TinyCC-era/post-musl ladder required before `bootstrap/binutils-tcc.ncl` can stop
being a placeholder. The parent task I2 is too large for a single drain step: it
requires dozens of chain-internal helper derivations before binutils 2.30 can
produce `as`, `ld`, `ar`, `ranlib`, `nm`, and `objcopy` for gcc-4.0.4.

## What Changes

- Add the missing derivations from the I1 inventory needed by
  `bootstrap/binutils-tcc.ncl`.
- Keep every new source, carried patch, and generated artifact pinned with
  URL/path, digest, and provenance at the first consuming derivation.
- Replace the `bootstrap/binutils-tcc.ncl` placeholder with a functional
  binutils 2.30 build using only prior chain outputs.
- Record validation transcripts proving no host compiler/libc/shell leakage,
  post-musl tool linkage against musl, and binutils assembler functionality.

## Scope

- **In scope**: `bzip2/coreutils/oyacc/bash`, tcc/musl rebuilds, post-musl
  `grep/sed/bzip2/m4/heirloom/flex/bison`, diffutils/coreutils/gawk,
  Perl/autoconf/automake/libtool ladder, and binutils 2.30.
- **Out of scope**: gcc-4.0.4 and later GCC stages; those remain in the parent
  or later scoped changes.

## Task Traceability

Implementation tasks split the ladder by epoch: early tcc-hosted utilities,
first musl/tcc rebuilds, post-musl text/parser tools,
Perl/autoconf/automake/libtool, and final binutils 2.30. Validation tasks map to
source pins, no-host leakage, post-musl linkage, binutils smoke output, and final
OpenSpec gates.

## Evidence Needed

Completion requires a `bootstrap/binutils-tcc.ncl` output whose transcript
records command, provider selection, exit status, output path, fallback status,
and placeholder rejection result. The evidence must include a source-pin audit,
post-musl linkage checks for `m4`, `flex`, `bison`, and `grep`, a no-host-leakage
audit, and an assembler smoke test producing a valid ELF object.
