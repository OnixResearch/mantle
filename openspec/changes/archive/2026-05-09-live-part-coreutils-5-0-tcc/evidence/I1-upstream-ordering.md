# I1 upstream ordering and output contract evidence

Task-ID: I1
Covers: bootstrap.part.coreutils.5.0.tcc

- Upstream part: fosslinux/live-bootstrap `coreutils 5.0` tcc build.
- Crunch derivation: `bootstrap/coreutils-5.0-tcc.ncl`.
- Direct predecessor imports observed in derivation: `stage0-posix`, `mes`, `tinycc-0.9.27`, `make-3.82-tcc`, `sed-4.0.9-tcc`, `tar-1.12-tcc`, and `bzip2-1.0.8-tcc`.
- Output contract after hardening: executable utilities `cat chmod cp echo install ln ls mkdir mv rm rmdir sort test true false head tail wc basename dirname tr uniq expr tee touch [` and a small mkdir/echo/cp/cat/test/[ smoke.
