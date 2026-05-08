# I1 upstream ordering and output contract evidence

Task-ID: I1
Covers: bootstrap.part.autoconf.2.61

- Upstream `parts.rst` treats `autoconf 2.61` as an independent live-bootstrap part; the OpenSpec proposal references that section.
- Crunch part identity is `bootstrap/autoconf-2.61.ncl`, output name `autoconf-2.61`, and source name `autoconf-2.61-src`.
- Direct predecessor imports in `bootstrap/autoconf-2.61.ncl`: `stage0-posix.ncl`, `make-tcc.ncl`, `sed-4.0.9-musl.ncl`, `m4-1.4.7-musl.ncl`, `perl-5.6.2-musl.ncl`, `coreutils-6.10-musl.ncl`, `gawk-3.0.4-musl.ncl`, `grep-2.4-musl.ncl`, `diffutils-2.7-musl.ncl`, `bash-2.05b-tcc.ncl`, and predecessor `automake-1.8.5.ncl`.
- Expected output contract hardened in this slice: `bin/autoconf`, `bin/autoreconf`, `bin/autoheader`, `bin/autom4te`, and `share/autoconf`.
