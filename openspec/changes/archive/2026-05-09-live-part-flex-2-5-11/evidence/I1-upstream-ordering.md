# I1 upstream ordering and output contract evidence

Task-ID: I1
Covers: bootstrap.part.flex.2.5.11

- Upstream part: fosslinux/live-bootstrap `flex 2.5.11`.
- Crunch derivation: `bootstrap/flex-2.5.11-musl.ncl`.
- Direct predecessor imports observed in derivation: `stage0-posix`, `tcc-0.9.27-musl-v2`, `musl-1.1.24-tcc-musl`, `make-3.82-tcc`, `sed-4.0.9-musl`, `m4-1.4.7-musl`, and `heirloom-devtools-070527`.
- Output contract after hardening: executable `bin/flex`, `bin/lex`, and an installed `flex --version` smoke.
