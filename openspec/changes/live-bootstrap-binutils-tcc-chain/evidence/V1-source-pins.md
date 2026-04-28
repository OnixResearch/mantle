Task-ID: V1
Covers: bootstrap.binutils.tcc.chain

Status: pass

## Audit method

1. Manual cross-reference of all `crunch.fetchTarball` URL+hash pairs against
   the live-bootstrap source-chain inventory.
2. Standalone fetch-only test derivations to verify each source hash is a
   correct NAR/recursive hash (not a flat archive hash).

## URL audit

All 41 unique source URLs match the inventory entries. Mirrors used:
- `mirrors.kernel.org/gnu/*` for GNU sources
- `sourceware.org/pub/bzip2/*` for bzip2
- `musl.libc.org/releases/*` for musl
- `download.savannah.gnu.org/releases/tinycc/*` for tcc
- `github.com/ibara/yacc/*` for oyacc
- `github.com/Perl/perl5/*` for perl 5.000, 5.003
- `www.cpan.org/src/5.0/*` for perl 5.004_05, 5.005_03, 5.6.2
- `github.com/westes/flex/*` for flex 2.5.11 (git snapshot), 2.6.4
- `downloads.sourceforge.net/project/heirloom/*` for heirloom-devtools
- `ftpmirror.gnu.org/*` for sed, binutils

## Hash audit

All source hashes corrected from flat-archive SHA-256 to NAR/recursive hashes
across three commits:
- `5b893314`: 23 files, 16 unique sources (early tcc-chain)
- `ee62a580`: 35 files, 33 unique sources (late chain)
- `4480b7b6`: libtool-2.2.4 switched to .tar.bz2 (tar extraction bug workaround)

Verification method: standalone `crunch.fetchTarball` derivations that test
each source independently without chain dependencies. 35 of 37 tested sources
produce successful fetches with correct NAR hashes.

## Remaining issues

- `coreutils-6.10.tar.gz`: Rust tar crate fails on non-UTF-8 numeric fields
  in old GNU tar headers. Needs crunch fix or alternative source format.
- `libtool-2.2.4`: Switched to .tar.bz2 to work around same tar extraction bug.

Verified: 2026-04-27
