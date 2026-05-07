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

This moves the active seam from a single header or single declaration to a TinyCC/Mes parser-state interaction across the combined generated Bison frontmatter: custom `yyoverflow`, `YYSTYPE`, declaration-specifier stack state, `SAVE_EXT_FLAGS`/`RESTORE_EXT_FLAGS`, and parser prototypes. Single-snippet normalization is not yet justified; the next diagnostic should bisect the combined manual frontmatter by adding the remaining groups one at a time to find the smallest multi-construct crashing set, then apply that exact source normalization to `bootstrap/gcc-4.0.ncl`.
