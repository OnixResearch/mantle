# V2 gcc-4.0 c-parse decl0 libtcc full-file va_start toggle

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured

## Command

- command: `timeout 590 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --json --store "$PWD/.crunch-drain/store" --no-substitute bootstrap/diag-gcc40-c-parse-boundary.ncl`
- exit-status: `1`
- report: `evidence/gcc40-cparse-decl0-libtcc-vastart-full-toggle-20260508/crunch-build-report.json`
- focused-log: `evidence/gcc40-cparse-decl0-libtcc-vastart-full-toggle-20260508/focused-build-log.txt`
- saved-derivation-log: `/home/brittonr/.local/state/crunch/logs/sm89p8xq97rnn10a8h9xlgcd35ph08w9-diag-gcc40-c-parse-boundary.drv.log`

## Focused results

| Probe | flags | rc |
|---|---|---:|
| `libtcc_vastart_full_all_va_start_noop` | `common` | 139 |
| `libtcc_vastart_full_strcat_va_start_noop` | `common` | 139 |
| `libtcc_vastart_full_error1_va_start_noop` | `common` | 139 |
| `libtcc_vastart_full_all_va_start_noop` | `one_source` | 139 |
| `libtcc_vastart_full_strcat_va_start_noop` | `one_source` | 139 |
| `libtcc_vastart_full_error1_va_start_noop` | `one_source` | 139 |

- direct decl0 runtime markers: `0`
- instrumented compiler build rc: `139`

## Interpretation

This full-file `libtcc.c` diagnostic tests whether the remaining predecessor compile crash is cleared by removing `va_start(ap, fmt)` callsites. It does not clear the full-file compile crash: removing all `va_start` callsites, only `strcat_printf`'s `va_start`, or only post-`error1` `va_start` callsites all still return `rc=139` under both common and `ONE_SOURCE=1` flags.

This means the prefix-only `__va_start(...)` trigger is real, but the full-file `libtcc.c` crash is not repaired by naive `va_start` removal alone. The remaining full-file context still contains another predecessor-sensitive path or requires a more precise varargs/runtime normalization. Direct `decl0` runtime execution remains unclaimed because no `diag-tcc-decl0-runtime:` markers emitted and the final instrumented compiler build still returns `rc=139` in this diagnostic.
