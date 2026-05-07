# GCC 4.0 c-parse decl0 runtime trace attempt

Task-ID: V2
Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
Status: captured-blocked

## Result
- Added a guarded diagnostic path that copies TinyCC 0.9.27 source, injects `tccgen.c` `decl0` branch markers, and attempts to build `/tmp/tcc-decl0-instrumented`.
- The instrumented compiler build itself segfaulted: `diag-tcc-decl0: build rc=139`.
- Because the instrumented compiler was unavailable, no `diag-tcc-decl0-runtime:*` markers executed for the probe matrix.
- Existing controls still reproduced: semicolon declarations `rc=0`; EOF/malformed declaration paths `rc=139`.

## Key excerpt
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
diag-tcc-decl0: build instrumented compiler
diag-tcc-decl0: build rc=139
diag-tcc-decl0: build-stderr: Segmentation fault (core dumped)
diag-tcc-decl0: instrumented compiler unavailable
diag-cparse: cparse_plain_decl_no_config rc=0
diag-cparse: cparse_plain_global_no_config rc=0
diag-cparse: cparse_plain_func_no_config rc=139
diag-cparse: cparse_var_prefix_config_only rc=139
diag-cparse: cparse_var_semicolon_config_only rc=0
diag-cparse: cparse_bad_init_semicolon_config_only rc=139
diag-cparse: cparse_bad_param_semicolon_config_only rc=139
diag-cparse: cparse_bad_comma_semicolon_config_only rc=139
```
