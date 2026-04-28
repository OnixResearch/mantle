# V6 Source Pin Audit Evidence

Timestamp: 2026-04-27T20:00:00Z

## Command

```
scripts/check-bootstrap-source-pins.rs bootstrap/*.ncl
```

## Result

```
source-pin audit: 78 files, 84 fetch blocks, 1 issues
```

## Issues

1. `bootstrap/seed-legacy.ncl:57` — `fetchTarball hash is not SRI format: 'raw_hash'`
   - **False positive**: seed-legacy.ncl uses Nickel variable indirection for the hash field
   - The actual hash value is bound at runtime from the `raw_hash` variable above

## Summary

All 83 real fetch blocks across 77 bootstrap .ncl files have valid SRI-format
hashes (`sha256-<base64>`), required fields (url, hash, name; +rev for fetchGit),
and consistent naming. The single reported issue is a known false positive from
variable indirection in seed-legacy.ncl.

Hash corrections applied in this session:
- Round 1: 16 unique source hashes across 23 files (early chain: patch, tcc, bzip2, make, sed, etc.)
- Round 2: 33 unique source hashes across 35 files (autoconf, automake, binutils, bison, flex, gcc, gmp, mpc, mpfr, musl, perl)
- Round 3: 2 tar format fixes (coreutils-6.10: .tar.lzma->.tar.gz, libtool-2.2.4: .tar.lzma->.tar.bz2)
