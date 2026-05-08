# V2 gcc40 c-parse decl0 libtcc full-file va_start/va_end cleanup toggle

Task-ID: V2 focused TinyCC `libtcc.c` full-file paired `va_start`/`va_end` diagnostic for GCC 4.0 c-parse decl0 predecessor compile boundary.
Covers: bootstrap.gcc40.runtime-validation

## Command

```sh
timeout 590 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --json --store "$PWD/.crunch-drain/store" --no-substitute bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Exit status: 1 (diagnostic derivation intentionally returns nonzero while the instrumented TinyCC build remains blocked).

## Evidence

- Run directory: `evidence/gcc40-cparse-decl0-libtcc-vastart-end-full-20260508/`
- Focused log: `evidence/gcc40-cparse-decl0-libtcc-vastart-end-full-20260508/focused-build-log.txt`
- Crunch derivation log: `/home/brittonr/.local/state/crunch/logs/hablkbzzggdkmimf68rwg4mg3qbzma2v-diag-gcc40-c-parse-boundary.drv.log`

## Focused results

| Variant | Flags | rc |
| --- | --- | --- |
| `libtcc_vastart_end_full_all_pairs` | `common` | `139` |
| `libtcc_vastart_end_full_strcat_pair` | `common` | `139` |
| `libtcc_vastart_end_full_error_pairs` | `common` | `139` |
| `libtcc_vastart_end_full_all_va_end_only` | `common` | `139` |
| `libtcc_vastart_end_full_all_pairs` | `one_source` | `139` |
| `libtcc_vastart_end_full_strcat_pair` | `one_source` | `139` |
| `libtcc_vastart_end_full_error_pairs` | `one_source` | `139` |
| `libtcc_vastart_end_full_all_va_end_only` | `one_source` | `139` |

- Direct `diag-tcc-decl0-runtime:` markers emitted: 0
- Instrumented TinyCC build rc: 139

## Interpretation

Unlike the minimal prefix, full-file `libtcc.c` is not repaired by removing paired `va_start`/`va_end` statements. Removing all pairs, only the `strcat_printf` pair, only the later error-family pairs, or only all `va_end` statements leaves `libtcc.c` at `rc=139` under both common and `ONE_SOURCE=1` flags. This preserves the prefix finding as a local source-shape trigger but shows the full-file predecessor compile crash has an additional context-sensitive trigger beyond paired varargs setup/cleanup statements. Direct `decl0` runtime marker claims remain out of scope until the instrumented TinyCC build succeeds and emits markers.
