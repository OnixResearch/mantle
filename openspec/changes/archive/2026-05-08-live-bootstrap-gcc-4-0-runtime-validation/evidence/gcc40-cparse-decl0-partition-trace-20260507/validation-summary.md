# Validation summary

- Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
- One-source preprocess: rc `0`, lines `19823`
- Full instrumented compiler build rc: `139`
- Component blocker: `tccgen.c` and `libtcc.c` compile with `rc=139`; `tccpp.c`, `tccelf.c`, and `x86_64-gen.c` compile with `rc=0`.
- Runtime decl0 markers: not reached.
