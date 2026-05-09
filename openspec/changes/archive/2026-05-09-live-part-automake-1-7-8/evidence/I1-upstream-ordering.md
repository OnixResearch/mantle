# I1 upstream ordering and output contract evidence

Task-ID: I1
Covers: bootstrap.part.automake.1.7.8

- Upstream part: fosslinux/live-bootstrap `automake 1.7.8`.
- Crunch derivation: `bootstrap/automake-1.7.8.ncl`.
- Direct predecessor imports observed in derivation: `make-3.82-tcc`, `sed-4.0.9-musl`, `perl-5.6.2-musl`, `coreutils-6.10-musl`, `gawk-3.0.4-musl`, `grep-2.4-musl`, `bash-2.05b-tcc`, `autoconf-2.55`, `stage0-posix`.
- Output contract after hardening: `bin/automake`, `bin/automake-1.7`, `bin/aclocal`, `bin/aclocal-1.7`, and `share/automake-1.7` must exist.
