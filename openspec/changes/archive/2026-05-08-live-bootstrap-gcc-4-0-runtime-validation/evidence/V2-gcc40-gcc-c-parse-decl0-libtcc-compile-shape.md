# V2 gcc40 c-parse decl0 libtcc tcc_compile body-shape diagnostic

Task-ID: V2 focused TinyCC `libtcc.c::tcc_compile` body-shape diagnostic for GCC 4.0 c-parse decl0 predecessor compile boundary.
Covers: bootstrap.gcc40.runtime-validation

## Command

```sh
timeout 590 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --json --store "$PWD/.crunch-drain/store" --no-substitute bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Exit status: 1 (diagnostic derivation intentionally returns nonzero while the instrumented TinyCC build remains blocked).

## Evidence

- Run directory: `evidence/gcc40-cparse-decl0-libtcc-compile-shape-20260508/`
- Focused log: `evidence/gcc40-cparse-decl0-libtcc-compile-shape-20260508/focused-build-log.txt`
- Crunch derivation log: `/home/brittonr/.local/state/crunch/logs/xm5ag60l2a8q93mj0kg3cqzbx7gg6glg-diag-gcc40-c-parse-boundary.drv.log`

## Focused results

| Variant | Flags | rc |
| --- | --- | --- |
| `libtcc_compile_shape_empty` | `common` | `139` |
| `libtcc_compile_shape_decls` | `common` | `139` |
| `libtcc_compile_shape_init_preamble` | `common` | `139` |
| `libtcc_compile_shape_begin_file` | `common` | `139` |
| `libtcc_compile_shape_setjmp_empty` | `common` | `139` |
| `libtcc_compile_shape_setjmp_state` | `common` | `139` |
| `libtcc_compile_shape_full` | `common` | `139` |
| `libtcc_compile_shape_empty` | `one_source` | `139` |
| `libtcc_compile_shape_decls` | `one_source` | `139` |
| `libtcc_compile_shape_init_preamble` | `one_source` | `139` |
| `libtcc_compile_shape_begin_file` | `one_source` | `139` |
| `libtcc_compile_shape_setjmp_empty` | `one_source` | `139` |
| `libtcc_compile_shape_setjmp_state` | `one_source` | `139` |
| `libtcc_compile_shape_full` | `one_source` | `139` |

- Direct `diag-tcc-decl0-runtime:` markers emitted: 0
- Instrumented TinyCC build rc: 139

## Interpretation

The balanced `tcc_compile` body-shape matrix finds the first failing synthesized variant at `empty` under `common` flags (`rc=139`). This localizes the second `libtcc.c` source-context trigger inside `tcc_compile` rather than in later full-file context. Direct `decl0` runtime marker claims remain out of scope until the instrumented TinyCC build succeeds and emits markers.
