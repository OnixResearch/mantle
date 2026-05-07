# V2 GCC 4.0 c-parse malformed-but-terminated declaration reduction

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured
Date: 2026-05-07
Diagnostic target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
Evidence dir: `evidence/gcc40-cparse-malformed-terminated-20260507/`

## Scope

This continues the declaration-termination reduction after commit `2819820c`.
It compares known crashing unterminated declarator prefixes with valid
semicolon-terminated declarations and malformed semicolon-bearing declarations.

The active diagnostic matrix also drops older frontmatter/full-source probe calls
that are already preserved in earlier evidence. The previous run with the new
small probes crossed the host argument-size limit before the derivation could
launch; trimming the already-recorded broad probes restored target execution.

## Result

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

Runtime reached `diag-gcc40-c-parse-boundary` and emitted per-probe output.
Valid semicolon-terminated function/variable/pointer declarations pass (`rc=0`),
while unterminated declarator prefixes return `rc=139`. Malformed declarations
that include a semicolon (`= ;`, bad parameter list, dangling comma) also return
`rc=139`, so the seam is broader than EOF-only handling: the declaration parser's
error path for incomplete/malformed declarators is unsafe, while valid declaration
termination is safe.

## Verification

- `bootstrap validate bootstrap/diag-gcc40-c-parse-boundary.ncl --resume`
  exited `1` because the diagnostic preserves failing `rc=139` probes.
- Normalized outputs: `probe-results.md`, `probe-results.json`,
  `validation-summary.md`, `validation-summary.json`, `doctor.json`,
  `build.stdout.log`, and `build.stderr.log`.
