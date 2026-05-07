# GCC 4.0 c-parse function-shape reduction results

- Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
- Evidence directory: `evidence/gcc40-cparse-function-shape-reduce-20260507`
- Runner status: `build-failed` / exit `1` (expected for rc=139 diagnostic probes)
- Diagnostic target reached: `True`

## Focused probe matrix

| Probe | rc |
|---|---:|
| `cparse_decl_after_config_only` | `0` |
| `cparse_global_after_config_only` | `0` |
| `cparse_oldstyle_func_after_config_only` | `139` |
| `cparse_int_func_after_config_only` | `139` |
| `cparse_func_empty_stmt_after_config_only` | `139` |
| `cparse_func_local_decl_after_config_only` | `139` |
| `cparse_static_func_after_config_only` | `139` |
| `cparse_extern_func_after_config_only` | `139` |
| `cparse_func_after_config_only` | `139` |
| `cparse_decl_after_system` | `0` |
| `cparse_global_after_system` | `0` |
| `cparse_oldstyle_func_after_system` | `139` |
| `cparse_int_func_after_system` | `139` |
| `cparse_func_after_system` | `139` |

## Interpretation

Declaration-only and global-only controls under `config-undef6.h` still pass.
Every config-only function-definition shape tested returns `139`, including
old-style `()`, `int ... { return 0; }`, empty-statement body, local-decl body,
`static`, `extern`, and the original `void ... (void) { }` form.

The seam is now below include/prelude and below the specific function spelling:
entry into function-definition parsing or its compound-body/finalization path under
`config-undef6.h`. Next reduction should split declaration grammar from function
body/open-brace/finalization behavior rather than adding more valid whole-function
variants.
