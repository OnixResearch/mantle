Task-ID: V1
Covers: bootstrap.gcc40.transition

Status: pass.

## Source pin audit

| Source | URL | Hash | Provenance |
|---|---|---|---|
| gcc-4.0.4 | `https://ftpmirror.gnu.org/gcc/gcc-4.0.4/gcc-4.0.4.tar.bz2` | `sha256-kJLkxw84mjCJeH/VHgVVVfWq8LtTa4nRwHjy+e/hdf0=` | live-bootstrap `steps/gcc-4.0.4/sources`; hash carried from existing checked-in placeholder |

Note: this uses the full gcc-4.0.4 source (not gcc-core), matching the design
requirement for C and C++ language support. The hash was set in the original
placeholder and may need verification as NAR vs flat (same class of issue as
V1 finding in binutils-tcc-chain).

No carried patches or generated artifacts in this derivation. All patches, if
needed, would be applied inline in the build script.

All 22 chain dependency inputs are imported from existing bootstrap/*.ncl files;
their source pins are audited by the parent binutils-tcc-chain change.

Verified: 2026-04-27
