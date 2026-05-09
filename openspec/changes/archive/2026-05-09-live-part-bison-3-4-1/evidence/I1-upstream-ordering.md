# I1 upstream ordering and output contract evidence

Task-ID: I1
Covers: bootstrap.part.bison.3.4.1

- Upstream part: fosslinux/live-bootstrap `bison 3.4.1`.
- Crunch derivation: `bootstrap/bison-3.4.1-musl.ncl`.
- Direct predecessor imports observed in derivation: `stage0-posix`, `tcc-0.9.27-musl-v2`, `musl-1.1.24-tcc-musl`, `make-3.82-tcc`, `sed-4.0.9-musl`, `m4-1.4.7-musl`, `flex-2.6.4-musl`, and `bison-2.3-musl`.
- Output contract after hardening: executable `bin/bison`, populated `share/bison` including `yacc.c`, and an installed `bison --version` smoke.
