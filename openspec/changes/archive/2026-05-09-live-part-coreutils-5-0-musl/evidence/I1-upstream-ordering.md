# I1 upstream ordering and output contract evidence

Task-ID: I1
Covers: bootstrap.part.coreutils.5.0.musl

- Upstream part: fosslinux/live-bootstrap `coreutils 5.0` musl rebuild.
- Crunch derivation: `bootstrap/coreutils-5.0-musl.ncl`.
- Direct predecessor imports observed in derivation: `stage0-posix`, `tcc-0.9.27-musl-v2`, `musl-1.1.24-tcc-musl`, `make-3.82-tcc`, and `sed-4.0.9-musl`.
- Output contract after hardening: executable utilities `cat chmod cp echo install ln ls mkdir mv rm rmdir sort test true false head tail wc basename dirname tr uniq expr tee touch [` and a small mkdir/echo/cp/cat/test/[ smoke.
