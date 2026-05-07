# V2 GCC 4.0 c-parse function declarator / semicolon reduction

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured
Date: 2026-05-07
Diagnostic target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
Evidence dir: `evidence/gcc40-cparse-function-declarator-semi-20260507/`

## Scope

This continues the function-prefix reduction after `cparse_plain_func_no_config`
and `cparse_func_prefix_declarator_config_only` returned `rc=139`. The slice
splits semicolon-terminated declarations from unterminated declarator prefixes
and compares function prefixes with nonfunction variable/pointer prefixes.

The diagnostic target was also trimmed to remove already-recorded broader
function-shape/system probes from the active matrix so the generated builder
script stays under the host argument-size limit; prior evidence notes retain
those results.

## Result

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

Runtime reached `diag-gcc40-c-parse-boundary` and emitted per-probe output.
Semicolon-terminated function declarations pass (`rc=0`) with and without
`config-undef6.h`, but unterminated declarator prefixes return `rc=139`.
The same `rc=139` appears for nonfunction variable/pointer declarator prefixes,
so the current seam is incomplete declaration/declarator termination at EOF, not
a function-specific body/open-brace parser action.

## Verification

- `bootstrap validate bootstrap/diag-gcc40-c-parse-boundary.ncl --resume`
  exited `1` because the diagnostic probes intentionally preserve failing
  `rc=139` cases.
- Normalized outputs: `probe-results.md`, `probe-results.json`,
  `validation-summary.md`, `validation-summary.json`, `doctor.json`,
  `build.stdout.log`, and `build.stderr.log`.
