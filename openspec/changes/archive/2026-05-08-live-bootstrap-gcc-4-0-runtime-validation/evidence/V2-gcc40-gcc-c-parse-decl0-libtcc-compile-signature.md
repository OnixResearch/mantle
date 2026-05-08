# V2 gcc40 c-parse decl0 libtcc tcc_compile signature diagnostic

Task-ID: V2 focused TinyCC `libtcc.c::tcc_compile` signature/name diagnostic for GCC 4.0 c-parse decl0 predecessor compile boundary.
Covers: bootstrap.gcc40.runtime-validation

## Command

```sh
timeout 590 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --json --store "$PWD/.crunch-drain/store" --no-substitute bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Exit status: 1 (diagnostic derivation intentionally returns nonzero while the instrumented TinyCC build remains blocked).

## Evidence

- Run directory: `evidence/gcc40-cparse-decl0-libtcc-compile-signature-20260508/`
- Focused log: `evidence/gcc40-cparse-decl0-libtcc-compile-signature-20260508/focused-build-log.txt`
- Crunch derivation log: `/home/brittonr/.local/state/crunch/logs/dv7nygpwc4sjzf8r52k3zw14by5zq34m-diag-gcc40-c-parse-boundary.drv.log`

## Focused results

| Variant | Flags | rc |
| --- | --- | --- |
| `libtcc_compile_signature_prototype_only` | `common` | `139` |
| `libtcc_compile_signature_original_empty` | `common` | `139` |
| `libtcc_compile_signature_renamed_empty` | `common` | `139` |
| `libtcc_compile_signature_nonstatic_empty` | `common` | `139` |
| `libtcc_compile_signature_void_param_empty` | `common` | `139` |
| `libtcc_compile_signature_void_return_empty` | `common` | `139` |
| `libtcc_compile_signature_c89_empty` | `common` | `139` |
| `libtcc_compile_signature_prototype_only` | `one_source` | `139` |
| `libtcc_compile_signature_original_empty` | `one_source` | `139` |
| `libtcc_compile_signature_renamed_empty` | `one_source` | `139` |
| `libtcc_compile_signature_nonstatic_empty` | `one_source` | `139` |
| `libtcc_compile_signature_void_param_empty` | `one_source` | `139` |
| `libtcc_compile_signature_void_return_empty` | `one_source` | `139` |
| `libtcc_compile_signature_c89_empty` | `one_source` | `139` |

- Direct `diag-tcc-decl0-runtime:` markers emitted: 0
- Instrumented TinyCC build rc: 139

## Interpretation

This matrix tests the empty-function trigger added after the known passing line-621 prefix by varying declaration-only, name, storage, return type, parameter style, and old-style C89 function syntax. The results distinguish whether the predecessor compile crash is tied to the exact `static int tcc_compile(TCCState *s1)` definition shape or to any added function definition in this context. Direct `decl0` runtime marker claims remain out of scope until the instrumented TinyCC build succeeds and emits markers.

Failing variants: `prototype_only`/common=rc139, `original_empty`/common=rc139, `renamed_empty`/common=rc139, `nonstatic_empty`/common=rc139, `void_param_empty`/common=rc139, `void_return_empty`/common=rc139, `c89_empty`/common=rc139, `prototype_only`/one_source=rc139, `original_empty`/one_source=rc139, `renamed_empty`/one_source=rc139, `nonstatic_empty`/one_source=rc139, `void_param_empty`/one_source=rc139 ....
