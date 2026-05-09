# I1 upstream ordering and output contract evidence

Task-ID: I1
Covers: bootstrap.part.autoconf.2.53

- Upstream part: fosslinux/live-bootstrap `autoconf 2.53`.
- Crunch derivation: `bootstrap/autoconf-2.53.ncl`.
- Direct predecessor imports observed in derivation: `make-3.82-tcc`, `sed-4.0.9-musl`, `m4-1.4.7-musl`, `perl-5.6.2-musl`, `coreutils-6.10-musl`, `gawk-3.0.4-musl`, `grep-2.4-musl`, `diffutils-2.7-musl`, `bash-2.05b-tcc`, `automake-1.6.3`, `stage0-posix`.
- Output contract after hardening: `bin/autoconf`, `bin/autoreconf`, `bin/autoheader`, `bin/autom4te`, and `share/autoconf` must exist.
