# V2 gcc-4.0 c-parse decl0 libtcc strcat va_start toggle

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured

## Command

- command: `timeout 590 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --json --store "$PWD/.crunch-drain/store" --no-substitute bootstrap/diag-gcc40-c-parse-boundary.ncl`
- exit-status: `1`
- report: `evidence/gcc40-cparse-decl0-libtcc-strcat-vastart-toggle-20260508/crunch-build-report.json`
- focused-log: `evidence/gcc40-cparse-decl0-libtcc-strcat-vastart-toggle-20260508/focused-build-log.txt`
- saved-derivation-log: `/home/brittonr/.local/state/crunch/logs/08wd7jsvw2v26w20n4g8z1d26pjl56xq-diag-gcc40-c-parse-boundary.drv.log`

## Focused results

| Probe | rc |
|---|---:|
| `libtcc_strcat_prefix_vastart_original` | 139 |
| `libtcc_strcat_prefix_vastart_static_empty` | 0 |
| `libtcc_strcat_prefix_vastart_static_va_start_macro` | 139 |
| `libtcc_strcat_prefix_vastart_nonstatic_va_start_macro` | 139 |
| `libtcc_strcat_prefix_vastart_static_frame_address_only` | 0 |
| `libtcc_strcat_prefix_vastart_static___va_start_frame` | 139 |
| `libtcc_strcat_prefix_vastart_static___va_start_zero` | 139 |
| `libtcc_strcat_prefix_vastart_static___va_start_fmt_addr` | 139 |

- direct decl0 runtime markers: `0`
- instrumented compiler build rc: `139`

## Interpretation

This prefix-only reduction splits the `va_start(ap, fmt)` trigger from the static-varargs slice. `static_empty` still compiles, but both static and nonstatic `va_start` macro variants return `rc=139`, so static linkage is no longer required once `va_start` is present.

The trigger is not `__builtin_frame_address(0)` alone: `static_frame_address_only` returns `rc=0`. The `__va_start(...)` call forms return `rc=139` whether the second argument is `__builtin_frame_address(0)`, `0`, or `(void *)&fmt`, narrowing the remaining predecessor compile crash to the x86_64 TinyCC/Mes `__va_start` call path used by `va_start`. Direct `decl0` runtime execution remains unclaimed because no `diag-tcc-decl0-runtime:` markers emitted and the instrumented compiler build still returns `rc=139`.
