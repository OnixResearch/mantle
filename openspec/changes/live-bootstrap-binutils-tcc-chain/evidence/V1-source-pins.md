Task-ID: V1
Covers: bootstrap.binutils.tcc.chain

Status: partial-pass. URLs match inventory; hash encoding issue found.

## Audit method

Manual cross-reference of all `crunch.fetchTarball` URL+hash pairs in the 46
new bootstrap/*.ncl files against the live-bootstrap source-chain inventory
at `openspec/changes/live-bootstrap-source-chain/evidence/I1-chain-inventory.md`.

Tool: base64/hex round-trip verification via shell + python3.

## URL audit

All 41 unique source URLs match the inventory entries exactly. Mirrors used:
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

### Correctly carried hashes (NAR/recursive, from existing working files)

These 5 source pins reuse the same tarball as an existing checked-in derivation
and carry the same NAR hash:

| Source | Carried from | Files using it |
|---|---|---|
| tcc-0.9.27 | `bootstrap/tinycc.ncl` | tcc-musl-prep.ncl, tcc-musl.ncl, tcc-musl-v2.ncl |
| sed-4.0.9 | `bootstrap/sed-tcc.ncl` | sed-4.0.9-musl.ncl |
| bzip2-1.0.8 | `bootstrap/bzip2-tcc.ncl` | bzip2-1.0.8-musl.ncl (same source) |
| coreutils-5.0 | `bootstrap/coreutils-5.0-tcc.ncl` | coreutils-5.0-musl.ncl (same source) |

### KNOWN ISSUE: flat archive hashes used where NAR hashes required

`crunch.fetchTarball` uses `mode = 'recursive` (NAR hash of unpacked tree).
The 39 new source pins were converted from the inventory's flat archive
SHA-256 hex digests to SRI format via `nix hash to-sri --type sha256 <hex>`.
These are flat archive hashes, NOT NAR/recursive hashes.

Evidence: the tcc-0.9.27 hash from the existing `tinycc.ncl`
(`sha256-3ija9gaKo/IKzXsGmFEzEaACxzBN1Im6+MhIv4KI7cw=`) differs from
the inventory hex converted to SRI
(`sha256-3iOvePypDOMt/y3UWzQysjNHQLubt7Bb9g/b/Dls65w=`), confirming
the inventory carries flat hashes while working derivation files carry NAR hashes.

**Impact**: builds will fail with hash mismatch on first `crunch build` for
all 39 sources with flat hashes. The error message will report the correct
NAR hash, enabling straightforward correction.

**Affected files** (39 sources with flat-archive hashes):
bzip2-tcc.ncl, coreutils-5.0-tcc.ncl, oyacc-tcc.ncl, bash-2.05b-tcc.ncl,
musl-1.1.24-tcc.ncl, musl-1.1.24-tcc-musl.ncl, grep-2.4-musl.ncl,
bzip2-1.0.8-musl.ncl, m4-1.4.7-musl.ncl, heirloom-devtools.ncl,
flex-2.5.11-musl.ncl, flex-2.6.4-musl.ncl, bison-2.3-musl.ncl,
bison-3.4.1-musl.ncl, diffutils-2.7-musl.ncl, coreutils-5.0-musl.ncl,
coreutils-6.10-musl.ncl, gawk-3.0.4-musl.ncl,
perl-5.000-musl.ncl, perl-5.003-musl.ncl, perl-5.004_05-musl.ncl,
perl-5.005_03-musl.ncl, perl-5.6.2-musl.ncl,
autoconf-2.52.ncl through autoconf-2.69.ncl (9 files),
automake-1.6.3.ncl through automake-1.15.1.ncl (8 files),
libtool-2.2.4.ncl, binutils-tcc.ncl.

**Unaffected files** (correct NAR hashes):
tcc-musl-prep.ncl, tcc-musl.ncl, tcc-musl-v2.ncl (carry tcc hash from tinycc.ncl),
sed-4.0.9-musl.ncl (carries sed hash from sed-tcc.ncl).

### Resolution path

Hash correction is blocked on V2 (requires `crunch build` to compute NAR
hashes). Each failed build will report the expected NAR hash in the error
message. Correction is mechanical: replace the flat SRI hash with the
reported NAR SRI hash.

Verified: 2026-04-27
