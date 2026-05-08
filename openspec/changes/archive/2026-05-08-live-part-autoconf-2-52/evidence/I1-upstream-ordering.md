# I1 upstream ordering and output contract evidence

Task-ID: I1
Covers: bootstrap.part.autoconf.2.52

- Upstream live-bootstrap tracks `autoconf 2.52` as the first Autoconf part in the chain; Crunch binds it to `bootstrap/autoconf-2.52.ncl`.
- Crunch part identity is output name `autoconf-2.52` and source name `autoconf-2.52-src`.
- Direct declared predecessors: `stage0-posix`, `tcc-0.9.27-musl-v2`, `musl-1.1.24-tcc-musl`, `make-3.82-tcc`, `sed-4.0.9-musl`, `m4-1.4.7-musl`, `perl-5.6.2-musl`, `coreutils-6.10-musl`, `gawk-3.0.4-musl`, `grep-2.4-musl`, `diffutils-2.7-musl`, and `bash-2.05b-tcc`.
- Expected output contract hardened in this slice: `bin/autoconf`, `bin/autoreconf`, `bin/autoheader`, `bin/autom4te`, and `share/autoconf`.
