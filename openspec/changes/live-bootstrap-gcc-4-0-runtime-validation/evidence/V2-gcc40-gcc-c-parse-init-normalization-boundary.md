# GCC 4.0 c-parse init normalization boundary

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured

Diagnostic target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-cparse-init-normalize-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-runtime-v4-20260507-state" \
  build --no-substitute bootstrap/diag-gcc40-c-parse-boundary.ncl \
  > openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-init-normalize-20260507/build.stdout.log \
  2> openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-init-normalize-20260507/build.stderr.log
```

Exit status: `1` (expected diagnostic builder failure after capturing the probe matrix).

Evidence bundle: `evidence/gcc40-cparse-init-normalize-20260507/`
Saved derivation log: `.crunch-drain/gcc40-runtime-v4-20260507-state/logs/ac9spk432aj6828adlycaxadf8c8pzg2-diag-gcc40-c-parse-boundary.drv.log`

## Result

The previous boundary held: declaration-only frontmatter remains valid, but any function definition in the generated c-parse frontmatter under the full GCC include stack segfaults TinyCC/Mes.

```text
diag-cparse: cparse_manual_src2_yystype_parser_no_init_func rc=0
diag-cparse: cparse_manual_src2_yystype_parser_lines_379 rc=0
diag-cparse: cparse_manual_src2_yystype_parser_lines_380 rc=139
```

Full-source normalization that removes the early `c_parse_init` body or delays it to the end of the translation unit does not advance the build:

```text
diag-cparse: cparse_full_config_undef6 rc=139
diag-cparse: cparse_full_config_undef6_no_init_func rc=139
diag-cparse: cparse_full_config_undef6_delayed_init_func rc=139
diag-cparse: cparse_full_config_undef6_void_newvs rc=139
diag-cparse: cparse_full_config_undef6_no_custom_yyoverflow rc=139
```

## Boundary

The failing full-source variants disprove the simple source-normalization hypothesis that `c_parse_init` placement alone is the active blocker. The smaller manual probes still identify the first reproducible trigger as a function definition after the generated GCC c-parse include/frontmatter stack, so the next high-ROI slice is a more reduced function-definition/codegen diagnostic under that exact include stack rather than more Bison-table or `c_parse_init` source movement in the full generated file.
