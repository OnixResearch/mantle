# GCC 4.0 c-parse reduction diagnostic

Focused command:

```sh
cargo run -- build --store .crunch-drain/gcc40-cparse-diag/store \
  bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Result: expected failure (`rc=1`) at the existing `gcc/c-parse.c` TinyCC segmentation boundary, after the diagnostic matrix ran.

Important reductions from `V2-gcc40-gcc-c-parse-reduction-build.diag.log`:

- `cparse_empty`: `rc=0`
- `cparse_inc_ansidecl`: `rc=0`
- `cparse_inc_autohost`: `rc=0`
- manually generated config equivalent (`auto-host.h` plus `ansidecl.h`): `rc=0`
- generated `config.h` include probe: `rc=139`
- `config.h` plus each later include prefix (`system.h`, `coretypes.h`, `tm.h`, `tree.h`, `langhooks.h`, `cpplib.h`, `c-tree.h`): `rc=139`
- full c-parse prefix reductions (`header_only`, `after_yytranslate`, `after_yyr1`, `after_yyr2`, `before_yytable`, `before_yyparse`) all remain `rc=139`
- full `c-parse.c`: `rc=139`

Interpretation: the next boundary is no longer the parser tables or `yyparse` body. The first reproducible frontend crash is the generated `build/gcc/config.h` include path under the real c-parse compile flags; the equivalent manual include wrapper does not crash. Next work should inspect/normalize generated `config.h` or the exact wrapper/output-path handling for `cparse_inc_config` before touching parser tables.
