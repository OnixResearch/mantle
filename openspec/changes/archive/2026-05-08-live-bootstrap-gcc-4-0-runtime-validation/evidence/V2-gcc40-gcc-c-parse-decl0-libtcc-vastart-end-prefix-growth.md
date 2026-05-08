# V2 gcc40 c-parse decl0 libtcc paired-cleanup prefix-growth diagnostic

Task-ID: V2 focused TinyCC `libtcc.c` paired `va_start`/`va_end` prefix-growth diagnostic for GCC 4.0 c-parse decl0 predecessor compile boundary.
Covers: bootstrap.gcc40.runtime-validation

## Command

```sh
timeout 590 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --json --store "$PWD/.crunch-drain/store" --no-substitute bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Exit status: 1 (diagnostic derivation intentionally returns nonzero while the instrumented TinyCC build remains blocked).

## Evidence

- Run directory: `evidence/gcc40-cparse-decl0-libtcc-vastart-end-prefix-growth-20260508/`
- Focused log: `evidence/gcc40-cparse-decl0-libtcc-vastart-end-prefix-growth-20260508/focused-build-log.txt`
- Crunch derivation log: `/home/brittonr/.local/state/crunch/logs/dmgpa0xd6xy09kw035wdqlgsd226m1xs-diag-gcc40-c-parse-boundary.drv.log`

## Focused results

| Variant | Flags | rc |
| --- | --- | --- |
| `libtcc_vastart_end_prefix_through_strcat_printf` | `common` | `0` |
| `libtcc_vastart_end_prefix_through_error1` | `common` | `0` |
| `libtcc_vastart_end_prefix_through_set_error_func` | `common` | `0` |
| `libtcc_vastart_end_prefix_through_error_noabort` | `common` | `0` |
| `libtcc_vastart_end_prefix_through_error` | `common` | `0` |
| `libtcc_vastart_end_prefix_through_warning` | `common` | `0` |
| `libtcc_vastart_end_prefix_through_open_bf` | `common` | `0` |
| `libtcc_vastart_end_prefix_through_close` | `common` | `0` |
| `libtcc_vastart_end_prefix_through_open` | `common` | `0` |
| `libtcc_vastart_end_prefix_through_strcat_printf` | `one_source` | `0` |
| `libtcc_vastart_end_prefix_through_error1` | `one_source` | `0` |
| `libtcc_vastart_end_prefix_through_set_error_func` | `one_source` | `0` |
| `libtcc_vastart_end_prefix_through_error_noabort` | `one_source` | `0` |
| `libtcc_vastart_end_prefix_through_error` | `one_source` | `0` |
| `libtcc_vastart_end_prefix_through_warning` | `one_source` | `0` |
| `libtcc_vastart_end_prefix_through_open_bf` | `one_source` | `0` |
| `libtcc_vastart_end_prefix_through_close` | `one_source` | `0` |
| `libtcc_vastart_end_prefix_through_open` | `one_source` | `0` |

- Direct `diag-tcc-decl0-runtime:` markers emitted: 0
- Instrumented TinyCC build rc: 139

## Interpretation

The paired-cleanup prefix-growth matrix tests complete `libtcc.c` prefixes after removing matched `va_start(ap, fmt)` and `va_end(ap)` lines. This distinguishes the local `strcat_printf` paired-varargs trigger from later source context that makes full-file paired cleanup still fail.

All tested balanced prefixes pass after paired cleanup; the remaining full-file trigger is later than the tested I/O-layer prefix window. Direct `decl0` runtime marker claims remain out of scope until the instrumented TinyCC build succeeds and emits markers.
