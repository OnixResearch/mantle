# GCC 4.0 c-parse system/prelude reduction results

- Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
- Evidence directory: `evidence/gcc40-cparse-system-prelude-reduce-20260507`
- Runner status: `build-failed` / exit `1` (expected for rc=139 diagnostic probes)
- Diagnostic target reached: `True`

## Focused probe matrix

| Probe | rc |
|---|---:|
| `cparse_decl_after_config_only` | `0` |
| `cparse_global_after_config_only` | `0` |
| `cparse_func_after_config_only` | `139` |
| `cparse_decl_after_system` | `0` |
| `cparse_global_after_system` | `0` |
| `cparse_oldstyle_func_after_system` | `139` |
| `cparse_int_func_after_system` | `139` |
| `cparse_func_before_system` | `139` |
| `cparse_func_after_system` | `139` |
| `cparse_func_after_coretypes` | `139` |
| `cparse_func_after_tm` | `139` |
| `cparse_func_after_tree` | `139` |
| `cparse_func_after_langhooks` | `139` |
| `cparse_func_after_input` | `139` |
| `cparse_func_after_cpplib` | `139` |
| `cparse_func_after_c_pragma` | `139` |
| `cparse_func_after_c_tree` | `139` |
| `cparse_func_after_flags` | `139` |
| `cparse_func_after_c_common` | `139` |

## Interpretation

The reduced matrix shows `system.h` is not required for the current crash:
`cparse_decl_after_config_only` and `cparse_global_after_config_only` pass, while
`cparse_func_after_config_only` returns `139`. Declarations/globals after
`system.h` also pass, while function-definition variants after `system.h` return
`139`.

Next reduction should target function-definition parsing itself under
`config-undef6.h`: declarator/parameter-list/body/compound-statement shape,
rather than more GCC include-stack bisection.
