# I1 upstream ordering and output contract evidence

Task-ID: I1
Covers: bootstrap.part.coreutils.6.10

- Upstream part: fosslinux/live-bootstrap `coreutils 6.10`.
- Crunch derivation: `bootstrap/coreutils-6.10-musl.ncl`.
- Direct predecessor imports observed in derivation: `stage0-posix`, `tcc-0.9.27-musl-v2`, `musl-1.1.24-tcc-musl`, `make-3.82-tcc`, `sed-4.0.9-musl`, `coreutils-5.0-musl`, `diffutils-2.7-musl`, and `bzip2-1.0.8-musl`.
- Output contract after hardening: executable utilities `cat chmod cp date echo install ln ls mkdir mktemp mv rm rmdir sort test true false head tail wc basename dirname tr uniq expr tee touch sha256sum [` and a small mkdir/echo/cp/cat/mktemp/sha256sum/test/[ smoke.
