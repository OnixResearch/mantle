# V2 GCC 4.0 c-parse front-matter boundary

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured

## Check result

The long `bootstrap/gcc-4.0.ncl` validation from `evidence/gcc40-gengtype-stub-20260507/` completed after the generated gengtype-header stub repair.  It progressed through the generated GCC helper phase (`genmodes`, `gengtype`, `genmddeps`, `genconstants`, `genpreds`, `genflags`, `gencheck`) and then failed at the real `c-parse.c` object compile:

```text
gcc40-cc ... /tmp/gcc-build/gcc-4.0.4/gcc/c-parse.c -o c-parse.o
make: *** [c-parse.o] Segmentation fault (core dumped)
```

This means the previous generated-header seam is cleared for the production build, but V2 remains open because the GCC 4.0 output is not built yet.

## Focused diagnostic

Two focused reruns under the same warm store/state captured the next seam:

- `evidence/gcc40-cparse-options2-20260507/`
- `evidence/gcc40-cparse-header-window-20260507/`

The diagnostic now explicitly builds `options.h` before c-parse probes.  Without that, `flags.h` probes were stale and failed only because the diagnostic had not generated the same header that production Make generates before `c-parse.o`.

Useful matrix excerpts:

```text
diag-cparse: cparse_undef6_inc_c_tree rc=0
diag-cparse: cparse_undef6_flags_balanced_lines_26 rc=0
diag-cparse: cparse_undef6_inc_flags rc=0
diag-cparse: cparse_undef6_inc_varray rc=0
diag-cparse: cparse_undef6_inc_output rc=0
diag-cparse: cparse_undef6_inc_toplev rc=0
diag-cparse: cparse_undef6_inc_ggc rc=0
diag-cparse: cparse_undef6_inc_c_common rc=0
diag-cparse: cparse_full_config_undef6 rc=139
diag-cparse: cparse_undef6_top_240 rc=0
diag-cparse: cparse_undef6_top_260 rc=0
diag-cparse: cparse_undef6_top_280 rc=139
diag-cparse: cparse_undef6_top_300 rc=139
diag-cparse: cparse_undef6_top_320 rc=139
diag-cparse: cparse_undef6_top_340 rc=139
diag-cparse: cparse_undef6_top_360 rc=0
diag-cparse: cparse_undef6_top_365 rc=139
diag-cparse: cparse_undef6_header_only rc=139
```

## Boundary

The include/header stack is no longer the active blocker once `options.h` is generated.  The remaining crash is in generated `c-parse.c` front matter after the top include block, around the parser declarations/macros between the `yyoverflow` macro and the `SAVE_EXT_FLAGS` / `RESTORE_EXT_FLAGS` extension-warning macros.

The non-monotonic prefix results mean malformed partial C windows are still present; do not overinterpret every failing prefix.  The durable facts are:

1. complete include-stack probes through `c-common.h` pass;
2. production `c-parse.o` still segfaults;
3. line-window probes show the next meaningful region is the generated parser front matter, not another generated GCC header.

## Next slice

Add a narrower diagnostic around c-parse lines 260-365 using balanced/manual snippets instead of raw prefixes, then test a source-normalization of the extension-warning save/restore macros only after a balanced snippet proves that exact construct is the crash trigger.
