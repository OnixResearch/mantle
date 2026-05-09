# I1 upstream ordering and output contract evidence

Task-ID: I1
Covers: bootstrap.part.binutils.2.30

- Upstream part: fosslinux/live-bootstrap `binutils 2.30`.
- Crunch derivation: `bootstrap/binutils-tcc.ncl`.
- Direct predecessor imports observed in derivation: `stage0-posix`, `mes`, `tcc-0.9.27-musl-v2`, `musl-1.1.24-tcc-musl`, `make-3.82-tcc`, `sed-4.0.9-musl`, `m4-1.4.7-musl`, `grep-2.4-musl`, `diffutils-2.7-musl`, `bash-2.05b-tcc`, `tar-1.12-tcc`, `bzip2-1.0.8-musl`.
- Output contract observed in derivation: executable `as`, `ld`, `ar`, `ranlib`, `nm`, and `objcopy`, plus assembler/archive/nm/objcopy/ld smoke artifacts.
