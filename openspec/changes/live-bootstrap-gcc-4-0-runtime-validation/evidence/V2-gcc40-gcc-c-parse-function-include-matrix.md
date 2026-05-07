# GCC 4.0 c-parse function include matrix runtime boundary

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured

Diagnostic target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`

## Command

```sh
PATH="/nix/store/dk9qhjgg469lv6mriys7v4c59igarmvx-bubblewrap-0.11.1/bin:$PATH"   ./target/debug/crunch --json     --store "$PWD/.crunch-drain/gcc40-runtime-v4-20260507-store"     --state-dir "$PWD/.crunch-drain/gcc40-runtime-v4-20260507-state"     build --no-substitute bootstrap/diag-gcc40-c-parse-boundary.ncl     > evidence/gcc40-cparse-function-include-matrix-20260507/build.stdout.log     2> evidence/gcc40-cparse-function-include-matrix-20260507/build.stderr.log
```

Evidence bundle: `evidence/gcc40-cparse-function-include-matrix-20260507/`

## Result

The warm-state diagnostic reached `diag-gcc40-c-parse-boundary` and emitted concrete `diag-cparse:` rows for the function-after-include matrix.

Captured matrix:

| Probe | Result |
|---|---:|
| `cparse_func_after_system` | `rc=139` |
| `cparse_func_after_coretypes` | `rc=139` |
| `cparse_func_after_tm` | `rc=139` |
| `cparse_func_after_tree` | `rc=139` |
| `cparse_func_after_langhooks` | `rc=139` |
| `cparse_func_after_input` | `rc=139` |
| `cparse_func_after_cpplib` | `rc=139` |
| `cparse_func_after_c_pragma` | `rc=139` |
| `cparse_func_after_c_tree` | `rc=139` |
| `cparse_func_after_flags` | `rc=139` |
| `cparse_func_after_c_common` | `rc=139` |

## Boundary

The first reduced function-definition failure is already `config-undef6.h` + `system.h` followed by a trivial `void cparse_function_probe (void) { }`. Extending the include chain through `c-common.h` does not change the result.

This supersedes the previous inconclusive timeout note for the same matrix: the function body parse crash is not caused by later `tm.h`, `tree.h`, `flags.h`, or `c-common.h` side effects. The next high-ROI slice is to reduce `system.h`/basic GCC prelude state versus the bare function-definition parser path.
