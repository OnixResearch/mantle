# V2 GCC 4.0 c-parse decl0 libtcc prefix-only strcat_printf signature toggle evidence

Diagnostic: `bootstrap/diag-gcc40-c-parse-boundary.ncl`.

Scope: predecessor TinyCC `libtcc.c` compile-boundary only. Prior full-translation-unit variants showed `strcat_printf` body, declaration/prototype shape, and `error1` body/callsite replacements still returned `rc=139`. This slice truncates `libtcc.c` immediately after the `strcat_printf` boundary and compares prefix-only declaration/definition variants, to separate the `strcat_printf` definition itself from later full-file context. No direct `decl0` runtime success is claimed unless runtime markers appear.

## Focused results

| stage | source | flags | rc |
|---|---|---:|---:|
| `libtcc_strcat_prefix_signature_original` | `libtcc-strcat-prefix-signature-original.c` | `common` | `139` |
| `libtcc_strcat_prefix_signature_prototype_only` | `libtcc-strcat-prefix-signature-prototype_only.c` | `common` | `0` |
| `libtcc_strcat_prefix_signature_fixed3_empty` | `libtcc-strcat-prefix-signature-fixed3_empty.c` | `common` | `0` |
| `libtcc_strcat_prefix_signature_fixed4_empty` | `libtcc-strcat-prefix-signature-fixed4_empty.c` | `common` | `0` |
| `libtcc_strcat_prefix_signature_nonstatic_varargs_empty` | `libtcc-strcat-prefix-signature-nonstatic_varargs_empty.c` | `common` | `0` |

## Findings

- Prefix-only variants captured: `libtcc_strcat_prefix_signature_original` rc=139, `libtcc_strcat_prefix_signature_prototype_only` rc=0, `libtcc_strcat_prefix_signature_fixed3_empty` rc=0, `libtcc_strcat_prefix_signature_fixed4_empty` rc=0, `libtcc_strcat_prefix_signature_nonstatic_varargs_empty` rc=0.
- Passing variants: `libtcc_strcat_prefix_signature_prototype_only` rc=0, `libtcc_strcat_prefix_signature_fixed3_empty` rc=0, `libtcc_strcat_prefix_signature_fixed4_empty` rc=0, `libtcc_strcat_prefix_signature_nonstatic_varargs_empty` rc=0.
- Failing variants: `libtcc_strcat_prefix_signature_original` rc=139.
- Instrumented compiler build remains unavailable (`build rc=139`), so this remains pre-marker evidence.
- No `diag-tcc-decl0-runtime:` markers emitted; direct `decl0` runtime validation remains blocked before marker execution.

## Artifacts

- Build report: `evidence/gcc40-cparse-decl0-libtcc-strcat-prefix-signature-toggle-20260508/crunch-build-report.json`
- Focused diagnostic log: `evidence/gcc40-cparse-decl0-libtcc-strcat-prefix-signature-toggle-20260508/focused-build-log.txt`
- Saved derivation log: `/home/brittonr/.local/state/crunch/logs/5hpayxmx42yripx56qlvh0yjfc8kmm7q-diag-gcc40-c-parse-boundary.drv.log`
