# V2 gcc40 c-parse decl0 libtcc va_start prefix-growth toggle

Task-ID: V2 focused TinyCC `libtcc.c` prefix-growth after `va_start` neutralization for GCC 4.0 c-parse decl0 predecessor compile boundary.
Covers: bootstrap.gcc40.runtime-validation

## Command

```sh
timeout 590 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --json --store "$PWD/.crunch-drain/store" --no-substitute bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Exit status: 1 (diagnostic derivation intentionally returns nonzero while the instrumented TinyCC build remains blocked).

## Evidence

- Run directory: `evidence/gcc40-cparse-decl0-libtcc-vastart-prefix-growth-20260508/`
- Focused log: `evidence/gcc40-cparse-decl0-libtcc-vastart-prefix-growth-20260508/focused-build-log.txt`
- Crunch derivation log: `/home/brittonr/.local/state/crunch/logs/20670qbwkdhy7pjbp286i8hvcl89d9r0-diag-gcc40-c-parse-boundary.drv.log`

## Focused results

| Variant | Flags | rc |
| --- | --- | --- |
| `libtcc_vastart_prefix_strcat_end_no_va_start` | `common` | `139` |
| `libtcc_vastart_prefix_error1_end_no_va_start` | `common` | `139` |
| `libtcc_vastart_prefix_set_error_func_end_no_va_start` | `common` | `139` |
| `libtcc_vastart_prefix_error_noabort_end_no_va_start` | `common` | `139` |
| `libtcc_vastart_prefix_error_end_no_va_start` | `common` | `139` |
| `libtcc_vastart_prefix_warning_end_no_va_start` | `common` | `139` |
| `libtcc_vastart_prefix_io_open_end_no_va_start` | `common` | `139` |
| `libtcc_vastart_prefix_compile_end_no_va_start` | `common` | `139` |
| `libtcc_vastart_prefix_compile_string_end_no_va_start` | `common` | `139` |
| `libtcc_vastart_prefix_strcat_end_no_va_start` | `one_source` | `139` |
| `libtcc_vastart_prefix_error1_end_no_va_start` | `one_source` | `139` |
| `libtcc_vastart_prefix_set_error_func_end_no_va_start` | `one_source` | `139` |
| `libtcc_vastart_prefix_error_noabort_end_no_va_start` | `one_source` | `139` |
| `libtcc_vastart_prefix_error_end_no_va_start` | `one_source` | `139` |
| `libtcc_vastart_prefix_warning_end_no_va_start` | `one_source` | `139` |
| `libtcc_vastart_prefix_io_open_end_no_va_start` | `one_source` | `139` |
| `libtcc_vastart_prefix_compile_end_no_va_start` | `one_source` | `139` |
| `libtcc_vastart_prefix_compile_string_end_no_va_start` | `one_source` | `139` |

- Direct `diag-tcc-decl0-runtime:` markers emitted: 0
- Instrumented TinyCC build rc: 139

## Interpretation

After `va_start(ap, fmt)` callsites are neutralized in each balanced prefix, even the `strcat_printf`-only prefix remains `rc=139` under both common and `ONE_SOURCE=1` flags. This corrects the naive full-file-repair hypothesis at prefix scale too: the crash is not cleared by deleting the `va_start` statement alone, and the remaining source shape likely involves adjacent `va_list`/`va_end`/variadic-call context or the macro-expanded cleanup path. This keeps runtime `decl0` marker claims out of scope until the instrumented TinyCC build succeeds and emits markers.
