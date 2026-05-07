# GCC 4.0 c-parse TinyCC source trace

- Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
- Runtime: reached diagnostic target; runner exits build-failed as expected for boundary probes.
- Scope: print TinyCC 0.9.27 `tccgen.c` declaration/parser anchors and focused source windows, then rerun declaration/error probes.

## TinyCC parser anchors
- `tccgen.c:80` `static int parse_btype(CType *type, AttributeDef *ad);`
- `tccgen.c:81` `static CType *type_decl(CType *type, AttributeDef *ad, int *v, int td);`
- `tccgen.c:84` `static void decl_initializer(CType *type, Section *sec, unsigned long c, int first, int size_only);`
- `tccgen.c:86` `static void decl_initializer_alloc(CType *type, AttributeDef *ad, int r, int has_init, int v, int scope);`
- `tccgen.c:87` `static void decl(int l);`
- `tccgen.c:88` `static int decl0(int l, int is_for_loop_init, Sym *);`
- `tccgen.c:99` `static void skip_or_save_block(TokenString **str);`
- `tccgen.c:4001` `static int parse_btype(CType *type, AttributeDef *ad)`
- `tccgen.c:4255` `static int post_type(CType *type, AttributeDef *ad, int storage, int td)`
- `tccgen.c:4401` `static CType *type_decl(CType *type, AttributeDef *ad, int *v, int td)`
- `tccgen.c:6241` `static void skip_or_save_block(TokenString **str)`
- `tccgen.c:6635` `static void decl_initializer(CType *type, Section *sec, unsigned long c, `
- `tccgen.c:6818` `static void decl_initializer_alloc(CType *type, AttributeDef *ad, int r, `
- `tccgen.c:7054` `static void gen_function(Sym *sym)`
- `tccgen.c:7144` `static int decl0(int l, int is_for_loop_init, Sym *func_sym)`
- `tccgen.c:7381` `static void decl(int l)`

## Focused phase trace
- `cparse_trace_valid_var_semicolon` source=`int cparse_trace_valid_var_probe;`
  - assembly: `60`
  - assembly stderr: `tcc: error: invalid option -- 'invalid option -- '%s''; tcc: error: invalid option -- 'invalid option -- '%s''`
  - preprocess: `139 preprocessed_lines=0`
  - preprocess stderr: `Segmentation fault (core dumped)`
  - preprocessed compile: `skipped`
- `cparse_trace_eof_var_prefix` source=`int cparse_trace_eof_var_probe`
  - assembly: `60`
  - assembly stderr: `tcc: error: invalid option -- 'invalid option -- '%s''; tcc: error: invalid option -- 'invalid option -- '%s''`
  - preprocess: `139 preprocessed_lines=0`
  - preprocess stderr: `Segmentation fault (core dumped)`
  - preprocessed compile: `skipped`
- `cparse_trace_bad_init_semicolon` source=`int cparse_trace_bad_init_probe = ;`
  - assembly: `60`
  - assembly stderr: `tcc: error: invalid option -- 'invalid option -- '%s''; tcc: error: invalid option -- 'invalid option -- '%s''`
  - preprocess: `139 preprocessed_lines=0`
  - preprocess stderr: `Segmentation fault (core dumped)`
  - preprocessed compile: `skipped`
- `cparse_trace_bad_comma_semicolon` source=`int cparse_trace_bad_comma_probe, ;`
  - assembly: `60`
  - assembly stderr: `tcc: error: invalid option -- 'invalid option -- '%s''; tcc: error: invalid option -- 'invalid option -- '%s''`
  - preprocess: `139 preprocessed_lines=0`
  - preprocess stderr: `Segmentation fault (core dumped)`
  - preprocessed compile: `skipped`

## Interpretation
- Runtime reached diag-gcc40-c-parse-boundary and printed TinyCC tccgen.c source anchors/windows.
- The relevant TinyCC parser path is parse_btype -> type_decl/post_type -> decl0, with decl0 branching from safe semicolon declarations to declaration error/function-body handling.
- Existing rc boundary is preserved: valid semicolon declarations pass, malformed/EOF declarator paths return rc139.
- Top-level -S/-E phase tracing remains unusable: -S returns rc60 with literal %s diagnostics and -E segfaults even for a valid semicolon declaration.
