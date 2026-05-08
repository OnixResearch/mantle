# GCC 4.0 c-parse frontmatter combination boundary

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured

Diagnostic target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-runtime-v4-20260507-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-runtime-v4-20260507-state" \
  bootstrap validate bootstrap/diag-gcc40-c-parse-boundary.ncl \
  --evidence-dir openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-trigger-split-20260507 \
  --resume
```

Follow-up probes were run in:

- `evidence/gcc40-cparse-combined-20260507/`
- `evidence/gcc40-cparse-combo-split-20260507/`
- `evidence/gcc40-cparse-trigger-split-20260507/`
- `evidence/gcc40-cparse-declspec-funcs-20260507/`
- `evidence/gcc40-cparse-group-bisect-20260507/`

## Result

The include stack is no longer the active boundary. All individual c-parse frontmatter constructs compile under the `config-undef6.h` header set:

```text
diag-cparse: cparse_preamble_bison_defines rc=0
diag-cparse: cparse_preamble_token_enum rc=0
diag-cparse: cparse_preamble_token_defines rc=0
diag-cparse: cparse_preamble_tokens_all rc=0
diag-cparse: cparse_manual_yystype rc=0
diag-cparse: cparse_manual_yystype_declspec rc=0
diag-cparse: cparse_manual_yystype_declspec_pushpop rc=0
diag-cparse: cparse_manual_yystype_save_restore rc=0
diag-cparse: cparse_manual_yyoverflow_void rc=0
diag-cparse: cparse_manual_yyoverflow_yystype_declspec rc=0
diag-cparse: cparse_manual_yyoverflow_declspec_pushpop rc=0
diag-cparse: cparse_manual_yyoverflow_save_restore rc=0
```

But their combined c-parse frontmatter still segfaults TinyCC/Mes:

```text
diag-cparse: cparse_manual_combined_frontmatter rc=139
diag-cparse: cparse_full_config_undef6 rc=139
diag-cparse: cparse_full_config_undef6_void_newvs rc=139
diag-cparse: cparse_full_config_undef6_no_line rc=139
diag-cparse: cparse_full_config_undef6_no_custom_yyoverflow rc=139
diag-cparse: cparse_full_config_undef6_declspec_funcs rc=139
```

## Boundary

The group-bisect follow-up narrowed the smallest manual frontmatter crash further than the prior combined-snippet result:

```text
diag-cparse: cparse_manual_src2_no_parser_protos rc=0
diag-cparse: cparse_manual_src2_yystype_parser_no_init_func rc=0
diag-cparse: cparse_manual_src2_yystype_parser_lines_379 rc=0
diag-cparse: cparse_manual_src2_yystype_parser_lines_380 rc=139
diag-cparse: cparse_manual_src2_yystype_init_func_only rc=139
diag-cparse: cparse_manual_func_empty rc=139
diag-cparse: cparse_manual_func_call_only rc=139
diag-cparse: cparse_manual_func_int_empty rc=139
diag-cparse: cparse_manual_yystype_init_decl_only rc=0
diag-cparse: cparse_manual_combined_frontmatter rc=139
```

The trigger is not `YYSTYPE`, custom `yyoverflow`, declaration-specifier state, or parser prototypes by themselves. Under the full GCC c-parse include stack, TinyCC/Mes segfaults as soon as a function definition appears in this frontmatter window: line 379 / prototypes only passes, line 380 (`void c_parse_init (void)`) starts failing, and even minimal `void c_parse_init (void) { }` and `int c_parse_init (void) { return 0; }` snippets return 139. A declaration-only `static void init_reswords (void);` remains valid.

This moves the active seam from a multi-construct Bison group interaction to a frontend parser/codegen bug for C function definitions after the generated GCC c-parse header stack. The next diagnostic should test source normalization that delays/removes the early `c_parse_init` function body (for example declaration-only in the frontmatter plus a later definition) and then rerun the full `c-parse.c` compile to see whether the full-source crash advances past this function-definition boundary.
