# V2 gcc40 c-parse decl0 libtcc strcat cleanup-prefix toggle

Task-ID: V2 focused TinyCC `libtcc.c` `strcat_printf` cleanup-prefix diagnostic for GCC 4.0 c-parse decl0 predecessor compile boundary.
Covers: bootstrap.gcc40.runtime-validation

## Command

```sh
timeout 590 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --json --store "$PWD/.crunch-drain/store" --no-substitute bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Exit status: 1 (diagnostic derivation intentionally returns nonzero while the instrumented TinyCC build remains blocked).

## Evidence

- Run directory: `evidence/gcc40-cparse-decl0-libtcc-strcat-cleanup-prefix-20260508/`
- Focused log: `evidence/gcc40-cparse-decl0-libtcc-strcat-cleanup-prefix-20260508/focused-build-log.txt`
- Crunch derivation log: `/home/brittonr/.local/state/crunch/logs/54ybxq74a4c5i9g8a1xa6qr7kixq1kc1-diag-gcc40-c-parse-boundary.drv.log`

## Focused results

| Variant | Flags | rc |
| --- | --- | --- |
| `libtcc_strcat_cleanup_prefix_original` | `common` | `139` |
| `libtcc_strcat_cleanup_prefix_no_va_start` | `common` | `139` |
| `libtcc_strcat_cleanup_prefix_no_va_end` | `common` | `139` |
| `libtcc_strcat_cleanup_prefix_no_va_start_no_va_end` | `common` | `0` |
| `libtcc_strcat_cleanup_prefix_no_call` | `common` | `139` |
| `libtcc_strcat_cleanup_prefix_va_list_only` | `common` | `0` |
| `libtcc_strcat_cleanup_prefix_empty_body` | `common` | `0` |
| `libtcc_strcat_cleanup_prefix_original` | `one_source` | `139` |
| `libtcc_strcat_cleanup_prefix_no_va_start` | `one_source` | `139` |
| `libtcc_strcat_cleanup_prefix_no_va_end` | `one_source` | `139` |
| `libtcc_strcat_cleanup_prefix_no_va_start_no_va_end` | `one_source` | `0` |
| `libtcc_strcat_cleanup_prefix_no_call` | `one_source` | `139` |
| `libtcc_strcat_cleanup_prefix_va_list_only` | `one_source` | `0` |
| `libtcc_strcat_cleanup_prefix_empty_body` | `one_source` | `0` |

- Direct `diag-tcc-decl0-runtime:` markers emitted: 0
- Instrumented TinyCC build rc: 139

## Interpretation

The minimal prefix matrix separates the remaining `strcat_printf` source-shape trigger from the earlier naive `va_start` hypothesis. Removing only `va_start`, only `va_end`, or only the `strcat_vprintf` call still leaves the prefix at `rc=139` under both common and `ONE_SOURCE=1` flags. Removing both `va_start` and `va_end` together changes the prefix to `rc=0`, and the `va_list`-only and empty-body controls also pass. This narrows the predecessor compile crash to the paired TinyCC/Mes varargs cleanup shape around `__va_start(...)` plus `va_end(...)`, not to `va_list` declaration alone, the call expression alone, or `va_start` removal alone. Direct `decl0` runtime marker claims remain out of scope until the instrumented TinyCC build succeeds and emits markers.
