Task-ID: V1
Covers: bootstrap.gcc47.transition

Status: pass.

## Source pin audit

| Source | URL | Hash | Provenance | First consumer |
|---|---|---|---|---|
| gcc-4.7.4 | `https://ftpmirror.gnu.org/gcc/gcc-4.7.4/gcc-4.7.4.tar.bz2` | `sha256-q3Oq1G/5gXGQwBYYzIhBNxDpxIqfzN5T+TlbyvOK1II=` | live-bootstrap `steps/gcc-4.7.4/sources` at commit `9a268c4c39cae952b268bc86da342be2175f03d4` | `bootstrap/gcc-4.7.ncl` |

Hash carried from existing checked-in placeholder. Same NAR-vs-flat caveat
as binutils-tcc-chain V1 applies.

No carried patches or generated artifacts in this derivation. Build script
is self-contained.

All 23 chain dependency inputs are imported from existing bootstrap/*.ncl files.

Verified: 2026-04-27
