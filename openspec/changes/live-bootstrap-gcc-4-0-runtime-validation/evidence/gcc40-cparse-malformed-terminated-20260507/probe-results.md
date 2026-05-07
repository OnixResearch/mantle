# GCC 4.0 c-parse malformed-but-terminated declaration reduction

| Probe | rc |
| --- | ---: |
| `cparse_plain_func_decl_semicolon_exact_no_config` | `0` |
| `cparse_plain_func_prefix_declarator_no_config` | `139` |
| `cparse_plain_oldstyle_prefix_no_config` | `139` |
| `cparse_plain_param_int_prefix_no_config` | `139` |
| `cparse_plain_var_prefix_no_config` | `139` |
| `cparse_plain_pointer_prefix_no_config` | `139` |
| `cparse_plain_var_semicolon_exact_no_config` | `0` |
| `cparse_plain_pointer_semicolon_exact_no_config` | `0` |
| `cparse_plain_bad_init_semicolon_no_config` | `139` |
| `cparse_plain_bad_param_semicolon_no_config` | `139` |
| `cparse_plain_bad_comma_semicolon_no_config` | `139` |
| `cparse_plain_decl_no_config` | `0` |
| `cparse_plain_global_no_config` | `0` |
| `cparse_plain_func_no_config` | `139` |
| `cparse_func_decl_semicolon_config_only` | `0` |
| `cparse_func_prefix_declarator_config_only` | `139` |
| `cparse_func_prefix_oldstyle_config_only` | `139` |
| `cparse_func_prefix_param_int_config_only` | `139` |
| `cparse_var_prefix_config_only` | `139` |
| `cparse_pointer_prefix_config_only` | `139` |
| `cparse_var_semicolon_config_only` | `0` |
| `cparse_pointer_semicolon_config_only` | `0` |
| `cparse_bad_init_semicolon_config_only` | `139` |
| `cparse_bad_param_semicolon_config_only` | `139` |
| `cparse_bad_comma_semicolon_config_only` | `139` |
| `cparse_decl_after_config_only` | `0` |
| `cparse_global_after_config_only` | `0` |
| `cparse_func_after_config_only` | `139` |

## Interpretation

- Runtime reached `diag-gcc40-c-parse-boundary` and emitted per-probe rc output.
- Unterminated declarator prefixes still return `rc=139`.
- Semicolon-terminated valid function/variable/pointer declarations pass (`rc=0`).
- Malformed semicolon-bearing declarations (`= ;`, bad parameter list, dangling comma) also return `rc=139`; the parser does not reach a clean diagnostic-error return for these malformed controls.
- This narrows the seam from function-body parsing to declaration parser error handling: valid declaration termination is safe, but both EOF-incomplete declarators and malformed terminated declarations enter a segfaulting error path.
