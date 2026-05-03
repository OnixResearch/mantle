## Context

The archived diagnostics proved `tcc26-i386` can compile several TinyCC 0.9.27 units and `-E -P` the full source, but segfaults on normal line-marker emission, `tccgen.c`, and full `ONE_SOURCE=1` compilation.

## Decisions

### 1. Treat line-marker output as a predecessor crash

**Choice:** Do not patch TinyCC 0.9.27 `tccpp.c::pp_line()` as the primary repair for `tcc27_preprocess_line_markers`.

**Rationale:** That diagnostic executes the already-built predecessor `tcc26-i386` preprocessor; rewriting the tcc27 source being compiled does not change the predecessor's line-marker formatter and would unnecessarily degrade the future tcc27 compiler source.

### 2. Keep reduction probes non-gating until the full handoff advances

**Choice:** Continue recording per-unit rc/stdout/stderr while making `tcc27_compile_object` the first gating handoff probe.

**Rationale:** This preserves evidence if one blocker is fixed and exposes the next compiler boundary without prematurely changing production bootstrap routing.

### 3. Reduce tccgen/full-source crashes in sibling copies

**Choice:** Add temporary copied-source reductions, such as bypassing `decl_initializer(...)`, only as named diagnostic probes.

**Rationale:** This distinguishes real handoff repairs from cut-down compiler experiments and prevents weakened compiler behavior from being mistaken for a production bootstrap path.
