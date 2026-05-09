# I1 upstream ordering and output contract evidence

Task-ID: I1
Covers: bootstrap.part.diffutils.2.7

- Upstream part: fosslinux/live-bootstrap `diffutils 2.7`.
- Crunch derivation: `bootstrap/diffutils-2.7-musl.ncl`.
- Direct predecessor imports observed in derivation: `stage0-posix`, `tcc-0.9.27-musl-v2`, `musl-1.1.24-tcc-musl`, `make-3.82-tcc`, and `sed-4.0.9-musl`.
- Output contract after hardening: executable `bin/diff` and `bin/cmp`, equal-file smoke, and different-file detection smoke.
