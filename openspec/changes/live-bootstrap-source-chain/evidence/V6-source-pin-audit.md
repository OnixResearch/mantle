Task-ID: V6
Covers: bootstrap.source.chain.implementation
Status: pass

## Source-Pin Audit Results

Command: `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/*.ncl`

Result: 78 files, 84 fetch blocks, 1 known false positive.

The single issue is `bootstrap/seed-legacy.ncl:57` which uses variable
indirection (`raw_hash`) instead of a literal SRI hash string. This is by
design - the legacy seed provider computes the hash at evaluation time.

All 83 other fetch blocks have:
- Required fields present (url, hash, name; +rev for fetchGit)
- Valid SRI-format hashes (sha256-<base64>)

## Hash Correction Summary

All source hashes were corrected from flat-archive SHA-256 to NAR/recursive
hashes across three commits:
- `5b893314`: 23 files, 16 unique sources (early tcc-chain)
- `ee62a580`: 35 files, 33 unique sources (late chain + gcc/gmp/mpfr/mpc/musl/binutils)
- `4480b7b6`: libtool-2.2.4 switched from .tar.gz to .tar.bz2 (tar extraction bug workaround)

Remaining issue: `coreutils-6.10.tar.gz` triggers a Rust tar crate bug with
non-UTF-8 numeric fields in old GNU tar headers. No .tar.bz2 alternative
exists for this version. Needs a crunch tar extraction fix.
