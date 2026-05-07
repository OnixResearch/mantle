# GCC 4.0 c-parse function declarator / semicolon reduction

| Probe | rc |
| --- | ---: |
| `cparse_plain_func_decl_semicolon_exact_no_config` | `0` |
| `cparse_plain_func_prefix_declarator_no_config` | `139` |
| `cparse_plain_oldstyle_prefix_no_config` | `139` |
| `cparse_plain_param_int_prefix_no_config` | `139` |
| `cparse_plain_var_prefix_no_config` | `139` |
| `cparse_plain_pointer_prefix_no_config` | `139` |
| `cparse_plain_decl_no_config` | `0` |
| `cparse_plain_global_no_config` | `0` |
| `cparse_plain_func_no_config` | `139` |
| `cparse_func_decl_semicolon_config_only` | `0` |
| `cparse_func_exact_config_only` | `139` |
| `cparse_func_prefix_declarator_config_only` | `139` |
| `cparse_func_prefix_oldstyle_config_only` | `139` |
| `cparse_func_prefix_param_int_config_only` | `139` |
| `cparse_var_prefix_config_only` | `139` |
| `cparse_pointer_prefix_config_only` | `139` |
| `cparse_func_prefix_open_brace_config_only` | `139` |
| `cparse_func_prefix_local_decl_config_only` | `139` |
| `cparse_decl_after_config_only` | `0` |
| `cparse_global_after_config_only` | `0` |
| `cparse_func_after_config_only` | `139` |

## Interpretation

- Runtime reached `diag-gcc40-c-parse-boundary` and emitted per-probe rc output.
- Exact semicolon-terminated function declarations pass (`rc=0`) with and without `config-undef6.h`.
- Unterminated function declarator prefixes return `rc=139` both without and with `config-undef6.h`.
- Unterminated nonfunction declarator prefixes (`int name`, `int *name`) also return `rc=139`, so the failing seam is not function-specific; it is incomplete declaration/declarator termination at EOF.
- Semicolon-terminated declaration/global controls continue to pass.
