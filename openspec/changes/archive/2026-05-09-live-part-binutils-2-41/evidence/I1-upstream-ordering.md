# I1 upstream ordering and output contract evidence

Task-ID: I1
Covers: bootstrap.part.binutils.2.41

- Upstream part: fosslinux/live-bootstrap `binutils 2.41`.
- Crunch derivation: `bootstrap/binutils-full.ncl`.
- Direct predecessor imports observed in derivation: `gcc-10.5.0`, `musl-1.2.5-full`, `binutils-2.30-tcc`, `make-3.82-tcc`, `bash-2.05b-tcc`, `coreutils-6.10-musl`, `sed-4.0.9-musl`, `m4-1.4.7-musl`, `flex-2.6.4-musl`, `bison-3.4.1-musl`, `perl-5.6.2-musl`, `grep-2.4-musl`, `diffutils-2.7-musl`, `gawk-3.0.4-musl`, `autoconf-2.69`, `automake-1.15.1`, `libtool-2.2.4`, `tar-1.12-tcc`, `stage0-posix`.
- Output contract after hardening: executable `as`, `ld`, `ar`, `ranlib`, `nm`, `objcopy`, `objdump`, `readelf`, and `strip`, plus assembler/archive/nm/objcopy/ld smoke artifacts.
