# GCC 4.0 c-parse decl0 native387 toggle trace

Task-ID: V2
Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
Status: captured

## Result
- Added a pre-marker toggle probe that disables the first `#if defined TCC_IS_NATIVE_387` block in `tccgen.c` before direct `decl0` marker insertion.
- `baseline` `tccgen.c` with `common` flags: rc=`139`
- `baseline` `libtcc.c` with `common` flags: rc=`139`
- `baseline` `tccgen.c` with `one_source` flags: rc=`139`
- `baseline` `libtcc.c` with `one_source` flags: rc=`139`
- `native387_disabled` `tccgen.c` with `common` flags: rc=`0`
- `native387_disabled` `libtcc.c` with `common` flags: rc=`139`
- `native387_disabled` `tccgen.c` with `one_source` flags: rc=`0`
- `native387_disabled` `libtcc.c` with `one_source` flags: rc=`139`
- Patched one-source preprocessing after the toggle remains rc=`0` with `19816` lines.
- Post-marker component `tccpp.c`: rc=`0`
- Post-marker component `tccgen.c`: rc=`0`
- Post-marker component `tccelf.c`: rc=`0`
- Post-marker component `x86_64-gen.c`: rc=`0`
- Post-marker component `libtcc.c`: rc=`139`
- Full instrumented compiler build remains rc=`139` before runtime markers execute.

## Conclusion
The `tccgen.c` predecessor compile segfault is isolated to the native x87/`TCC_IS_NATIVE_387` path: disabling that block changes `tccgen.c` from `rc=139` to `rc=0` under both common and `ONE_SOURCE=1` flags, and post-marker `tccgen.c` also compiles. The remaining instrumented compiler blocker is now `libtcc.c` (`rc=139`), so direct `decl0` runtime markers still do not execute.

## Excerpt
```text
diag-cparse: cparse_plain_func_decl_semicolon_exact_no_config rc=0
diag-cparse: cparse_plain_func_prefix_declarator_no_config rc=139
diag-cparse: cparse_plain_oldstyle_prefix_no_config rc=139
diag-cparse: cparse_plain_param_int_prefix_no_config rc=139
diag-cparse: cparse_plain_var_prefix_no_config rc=139
diag-cparse: cparse_plain_pointer_prefix_no_config rc=139
diag-cparse: cparse_plain_var_semicolon_exact_no_config rc=0
diag-cparse: cparse_plain_pointer_semicolon_exact_no_config rc=0
diag-cparse: cparse_plain_bad_init_semicolon_no_config rc=139
diag-cparse: cparse_plain_bad_param_semicolon_no_config rc=139
diag-cparse: cparse_plain_bad_comma_semicolon_no_config rc=139
diag-tcc-decl0-prepart: compile baseline tccgen.c flags=common
diag-tcc-decl0-prepart: compile baseline tccgen.c flags=common rc=139
diag-tcc-decl0-prepart: tccgen.c baseline common stderr: Segmentation fault (core dumped)
diag-tcc-decl0-prepart: compile baseline libtcc.c flags=common
diag-tcc-decl0-prepart: compile baseline libtcc.c flags=common rc=139
diag-tcc-decl0-prepart: libtcc.c baseline common stderr: Segmentation fault (core dumped)
diag-tcc-decl0-prepart: compile baseline tccgen.c flags=one_source
diag-tcc-decl0-prepart: compile baseline tccgen.c flags=one_source rc=139
diag-tcc-decl0-prepart: tccgen.c baseline one_source stderr: Segmentation fault (core dumped)
diag-tcc-decl0-prepart: compile baseline libtcc.c flags=one_source
diag-tcc-decl0-prepart: compile baseline libtcc.c flags=one_source rc=139
diag-tcc-decl0-prepart: libtcc.c baseline one_source stderr: Segmentation fault (core dumped)
diag-tcc-decl0-prepart: compile native387_disabled tccgen.c flags=common
diag-tcc-decl0-prepart: compile native387_disabled tccgen.c flags=common rc=0
diag-tcc-decl0-prepart: compile native387_disabled libtcc.c flags=common
diag-tcc-decl0-prepart: compile native387_disabled libtcc.c flags=common rc=139
diag-tcc-decl0-prepart: libtcc.c native387_disabled common stderr: Segmentation fault (core dumped)
diag-tcc-decl0-prepart: compile native387_disabled tccgen.c flags=one_source
diag-tcc-decl0-prepart: compile native387_disabled tccgen.c flags=one_source rc=0
diag-tcc-decl0-prepart: compile native387_disabled libtcc.c flags=one_source
diag-tcc-decl0-prepart: compile native387_disabled libtcc.c flags=one_source rc=139
diag-tcc-decl0-prepart: libtcc.c native387_disabled one_source stderr: Segmentation fault (core dumped)
diag-tcc-decl0-part: preprocess one-source
diag-tcc-decl0-part: preprocess rc=0 lines=19816
diag-tcc-decl0-part: compile tccpp.c
diag-tcc-decl0-part: compile tccpp.c rc=0
diag-tcc-decl0-part: compile tccgen.c
diag-tcc-decl0-part: compile tccgen.c rc=0
diag-tcc-decl0-part: compile tccelf.c
diag-tcc-decl0-part: compile tccelf.c rc=0
diag-tcc-decl0-part: compile x86_64-gen.c
diag-tcc-decl0-part: compile x86_64-gen.c rc=0
diag-tcc-decl0-part: compile libtcc.c
diag-tcc-decl0-part: compile libtcc.c rc=139
diag-tcc-decl0-part: libtcc.c stderr: Segmentation fault (core dumped)
diag-tcc-decl0: build instrumented compiler
diag-tcc-decl0: build rc=139
diag-tcc-decl0: build-stderr: Segmentation fault (core dumped)
diag-tcc-decl0: instrumented compiler unavailable
diag-cparse-trace: cparse_trace_valid_var_semicolon source-begin
diag-cparse-trace: cparse_trace_valid_var_semicolon src:001:int cparse_trace_valid_var_probe;
diag-cparse-trace: cparse_trace_valid_var_semicolon source-end
diag-cparse-trace: cparse_trace_valid_var_semicolon assembly_rc=60
diag-cparse-trace: cparse_trace_valid_var_semicolon assembly-stderr: tcc: error: invalid option -- 'invalid option -- '%s''
diag-cparse-trace: cparse_trace_valid_var_semicolon preprocess_rc=139 preprocessed_lines=0
diag-cparse-trace: cparse_trace_valid_var_semicolon preprocess-stderr: Segmentation fault (core dumped)
diag-cparse-trace: cparse_trace_valid_var_semicolon preprocessed_compile_rc=skipped
diag-cparse-trace: cparse_trace_eof_var_prefix source-begin
diag-cparse-trace: cparse_trace_eof_var_prefix src:001:int cparse_trace_eof_var_probe
diag-cparse-trace: cparse_trace_eof_var_prefix source-end
diag-cparse-trace: cparse_trace_eof_var_prefix assembly_rc=60
diag-cparse-trace: cparse_trace_eof_var_prefix assembly-stderr: tcc: error: invalid option -- 'invalid option -- '%s''
diag-cparse-trace: cparse_trace_eof_var_prefix preprocess_rc=139 preprocessed_lines=0
diag-cparse-trace: cparse_trace_eof_var_prefix preprocess-stderr: Segmentation fault (core dumped)
diag-cparse-trace: cparse_trace_eof_var_prefix preprocessed_compile_rc=skipped
diag-cparse-trace: cparse_trace_bad_init_semicolon source-begin
diag-cparse-trace: cparse_trace_bad_init_semicolon src:001:int cparse_trace_bad_init_probe = ;
diag-cparse-trace: cparse_trace_bad_init_semicolon source-end
diag-cparse-trace: cparse_trace_bad_init_semicolon assembly_rc=60
diag-cparse-trace: cparse_trace_bad_init_semicolon assembly-stderr: tcc: error: invalid option -- 'invalid option -- '%s''
diag-cparse-trace: cparse_trace_bad_init_semicolon preprocess_rc=139 preprocessed_lines=0
diag-cparse-trace: cparse_trace_bad_init_semicolon preprocess-stderr: Segmentation fault (core dumped)
diag-cparse-trace: cparse_trace_bad_init_semicolon preprocessed_compile_rc=skipped
diag-cparse-trace: cparse_trace_bad_comma_semicolon source-begin
diag-cparse-trace: cparse_trace_bad_comma_semicolon src:001:int cparse_trace_bad_comma_probe, ;
diag-cparse-trace: cparse_trace_bad_comma_semicolon source-end
diag-cparse-trace: cparse_trace_bad_comma_semicolon assembly_rc=60
diag-cparse-trace: cparse_trace_bad_comma_semicolon assembly-stderr: tcc: error: invalid option -- 'invalid option -- '%s''
diag-cparse-trace: cparse_trace_bad_comma_semicolon preprocess_rc=139 preprocessed_lines=0
diag-cparse-trace: cparse_trace_bad_comma_semicolon preprocess-stderr: Segmentation fault (core dumped)
diag-cparse-trace: cparse_trace_bad_comma_semicolon preprocessed_compile_rc=skipped
diag-cparse: cparse_plain_decl_no_config rc=0
diag-cparse: cparse_plain_global_no_config rc=0
diag-cparse: cparse_plain_func_no_config rc=139
diag-cparse: cparse_func_decl_semicolon_config_only rc=0
diag-cparse: cparse_func_exact_config_only rc=139
diag-cparse: cparse_func_prefix_declarator_config_only rc=139
diag-cparse: cparse_func_prefix_oldstyle_config_only rc=139
diag-cparse: cparse_func_prefix_param_int_config_only rc=139
```
