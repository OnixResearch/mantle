# Design: i386 TinyCC 0.9.27 source diagnostics

Keep this as a sibling diagnostic proof only. The production Make/TinyCC bootstrap route remains unchanged until the predecessor TinyCC handoff blocker is understood.

The proof records non-gating diagnostics before the gating `tcc27_compile_object` step:

- `tcc27_preprocess_no_lines`: proves preprocessing succeeds when line markers are suppressed with `-P`.
- `tcc27_preprocess_line_markers`: isolates the line-marker formatting/diagnostic segfault class.
- representative unit compiles (`tccpp.c`, `tccelf.c`, `i386-gen.c`, `libtcc.c`, `tccgen.c`): separates broad source parse/codegen health from the failing full `ONE_SOURCE=1` pass1 compile.

The gating behavior remains unchanged: if `tcc27_compile_object` fails, the derivation writes a blocked summary and exits successfully so the evidence bundle can be inspected.
