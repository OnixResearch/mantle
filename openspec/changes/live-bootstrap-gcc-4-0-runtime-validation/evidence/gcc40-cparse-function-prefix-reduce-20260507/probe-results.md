# GCC 4.0 c-parse function-prefix reduction results

- Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
- Evidence directory: `evidence/gcc40-cparse-function-prefix-reduce-20260507`
- Runner status: `build-failed` / exit `1` (expected for rc=139 diagnostic probes)
- Diagnostic target reached: `True`

## Focused probe matrix

| Probe | rc |
|---|---:|
| `cparse_plain_decl_no_config` | `0` |
| `cparse_plain_global_no_config` | `0` |
| `cparse_plain_func_no_config` | `139` |
| `cparse_func_exact_config_only` | `139` |
| `cparse_func_prefix_declarator_config_only` | `139` |
| `cparse_func_prefix_open_brace_config_only` | `139` |
| `cparse_func_prefix_local_decl_config_only` | `139` |
| `cparse_decl_after_config_only` | `0` |
| `cparse_global_after_config_only` | `0` |
| `cparse_func_after_config_only` | `139` |

## Interpretation

The crash does not require `config-undef6.h`: plain declaration and global probes
pass, while a plain valid function definition returns `139`. Under `config-undef6.h`,
the exact function with no trailing marker still returns `139`; even the incomplete
function-declarator prefix `void cparse_prefix_probe (void)` returns `139` before a
body or open brace is needed.

The next seam is earlier than compound-body/finalization: split the parser handling
of a function declarator from declaration termination (`;`) and compare against
nonfunction declarator prefixes before touching more include/header state.
