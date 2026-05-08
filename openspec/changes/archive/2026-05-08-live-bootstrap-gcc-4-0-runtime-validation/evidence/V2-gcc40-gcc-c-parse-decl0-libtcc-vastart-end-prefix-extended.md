# V2 gcc40 c-parse decl0 libtcc paired-cleanup extended-prefix diagnostic

Task-ID: V2 focused TinyCC `libtcc.c` extended paired `va_start`/`va_end` prefix-growth diagnostic for GCC 4.0 c-parse decl0 predecessor compile boundary.
Covers: bootstrap.gcc40.runtime-validation

## Command

```sh
timeout 590 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --json --store "$PWD/.crunch-drain/store" --no-substitute bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Exit status: 1 (diagnostic derivation intentionally returns nonzero while the instrumented TinyCC build remains blocked).

## Evidence

- Run directory: `evidence/gcc40-cparse-decl0-libtcc-vastart-end-prefix-extended-20260508/`
- Focused log: `evidence/gcc40-cparse-decl0-libtcc-vastart-end-prefix-extended-20260508/focused-build-log.txt`
- Crunch derivation log: `/home/brittonr/.local/state/crunch/logs/ici8plzsj4m8lb4qx8zlnbrmh02g65nn-diag-gcc40-c-parse-boundary.drv.log`

## Focused results

| Variant | Flags | rc |
| --- | --- | --- |
| `libtcc_vastart_end_prefix_through_compile` | `common` | `139` |
| `libtcc_vastart_end_prefix_through_compile_string` | `common` | `139` |
| `libtcc_vastart_end_prefix_through_define_symbol` | `common` | `139` |
| `libtcc_vastart_end_prefix_through_undefine_symbol` | `common` | `139` |
| `libtcc_vastart_end_prefix_through_cleanup` | `common` | `139` |
| `libtcc_vastart_end_prefix_through_new` | `common` | `139` |
| `libtcc_vastart_end_prefix_through_delete` | `common` | `139` |
| `libtcc_vastart_end_prefix_through_set_output_type` | `common` | `139` |
| `libtcc_vastart_end_prefix_through_add_include_path` | `common` | `139` |
| `libtcc_vastart_end_prefix_through_add_sysinclude_path` | `common` | `139` |
| `libtcc_vastart_end_prefix_through_compile` | `one_source` | `139` |
| `libtcc_vastart_end_prefix_through_compile_string` | `one_source` | `139` |
| `libtcc_vastart_end_prefix_through_define_symbol` | `one_source` | `139` |
| `libtcc_vastart_end_prefix_through_undefine_symbol` | `one_source` | `139` |
| `libtcc_vastart_end_prefix_through_cleanup` | `one_source` | `139` |
| `libtcc_vastart_end_prefix_through_new` | `one_source` | `139` |
| `libtcc_vastart_end_prefix_through_delete` | `one_source` | `139` |
| `libtcc_vastart_end_prefix_through_set_output_type` | `one_source` | `139` |
| `libtcc_vastart_end_prefix_through_add_include_path` | `one_source` | `139` |
| `libtcc_vastart_end_prefix_through_add_sysinclude_path` | `one_source` | `139` |

- Direct `diag-tcc-decl0-runtime:` markers emitted: 0
- Instrumented TinyCC build rc: 139

## Interpretation

The extended paired-cleanup prefix-growth matrix finds the second source-context trigger at `through_compile` under `common` flags (`rc=139`). The previous committed prefix-growth slice passed through `tcc_open` / line 621, so the unresolved full-file `libtcc.c` crash reappears when the next complete function, `tcc_compile` (lines 623-661), is added. Direct `decl0` runtime marker claims remain out of scope until the instrumented TinyCC build succeeds and emits markers.
