# V2 gcc-4.0 c-parse decl0 libtcc strcat static-varargs toggle

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured

## Command

- command: `timeout 590 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --json --store "$PWD/.crunch-drain/store" --no-substitute bootstrap/diag-gcc40-c-parse-boundary.ncl`
- exit-status: `1`
- report: `evidence/gcc40-cparse-decl0-libtcc-strcat-static-varargs-toggle-20260508/crunch-build-report.json`
- focused-log: `evidence/gcc40-cparse-decl0-libtcc-strcat-static-varargs-toggle-20260508/focused-build-log.txt`
- saved-derivation-log: `/home/brittonr/.local/state/crunch/logs/daqk396alfsild2jsnf8xz1x0lsg0aax-diag-gcc40-c-parse-boundary.drv.log`

## Focused results

| Probe | rc |
|---|---:|
| `libtcc_strcat_prefix_static_varargs_original` | 139 |
| `libtcc_strcat_prefix_static_varargs_static_empty` | 0 |
| `libtcc_strcat_prefix_static_varargs_static_va_list` | 0 |
| `libtcc_strcat_prefix_static_varargs_static_va_start` | 139 |
| `libtcc_strcat_prefix_static_varargs_static_va_start_end` | 139 |
| `libtcc_strcat_prefix_static_varargs_static_direct_call` | 0 |
| `libtcc_strcat_prefix_static_varargs_renamed_static_original` | 139 |

- direct decl0 runtime markers: `0`
- instrumented compiler build rc: `139`

## Interpretation

This prefix-only reduction refines the previous `strcat_printf` signature boundary. The crash does not require the helper name or the `strcat_vprintf` call by itself: a renamed static original still returns `rc=139`, while the static direct-call-with-uninitialized-`va_list` control returns `rc=0`.

The smallest newly isolated trigger in this matrix is the `va_start(ap, fmt)` path inside a static variadic prefix-only function: `static_empty` and `static_va_list` compile (`rc=0`), while `static_va_start` and `static_va_start_end` return `rc=139`. Direct `decl0` runtime execution remains unclaimed because no `diag-tcc-decl0-runtime:` markers emitted and the instrumented compiler build still returns `rc=139`.
