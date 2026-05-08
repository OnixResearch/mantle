# V2 GCC 4.0 c-parse decl0 libtcc strcat_printf signature/prototype toggle evidence

Diagnostic: `bootstrap/diag-gcc40-c-parse-boundary.ncl`.

Scope: predecessor TinyCC `libtcc.c` compile-boundary only. Prior evidence showed prefixes through `strcat_vprintf` pass, adding the `strcat_printf` definition flips to `rc=139`, and replacing only the helper body still leaves `rc=139`. This slice changes the `strcat_printf` declaration/definition shape while preserving the full `libtcc.c` translation unit. No direct `decl0` runtime success is claimed unless runtime markers appear.

## Focused results

| stage | source | flags | rc |
|---|---|---:|---:|
| `libtcc_strcat_signature_prototype_only` | `libtcc-strcat-signature-prototype_only.c` | `common` | `139` |
| `libtcc_strcat_signature_fixed3_empty` | `libtcc-strcat-signature-fixed3_empty.c` | `common` | `139` |
| `libtcc_strcat_signature_fixed4_empty` | `libtcc-strcat-signature-fixed4_empty.c` | `common` | `139` |
| `libtcc_strcat_signature_nonstatic_varargs_empty` | `libtcc-strcat-signature-nonstatic_varargs_empty.c` | `common` | `139` |
| `libtcc_strcat_signature_kr_fixed_empty` | `libtcc-strcat-signature-kr_fixed_empty.c` | `common` | `139` |

## Findings

- Signature/prototype variants captured: `libtcc_strcat_signature_prototype_only` rc=139, `libtcc_strcat_signature_fixed3_empty` rc=139, `libtcc_strcat_signature_fixed4_empty` rc=139, `libtcc_strcat_signature_nonstatic_varargs_empty` rc=139, `libtcc_strcat_signature_kr_fixed_empty` rc=139.
- Failing variants: `libtcc_strcat_signature_prototype_only` rc=139, `libtcc_strcat_signature_fixed3_empty` rc=139, `libtcc_strcat_signature_fixed4_empty` rc=139, `libtcc_strcat_signature_nonstatic_varargs_empty` rc=139, `libtcc_strcat_signature_kr_fixed_empty` rc=139.
- Instrumented compiler build remains unavailable (`build rc=139`), so this remains pre-marker evidence.
- No `diag-tcc-decl0-runtime:` markers emitted; direct `decl0` runtime validation remains blocked before marker execution.

## Artifacts

- Build report: `evidence/gcc40-cparse-decl0-libtcc-strcat-signature-toggle-20260508/crunch-build-report.json`
- Focused diagnostic log: `evidence/gcc40-cparse-decl0-libtcc-strcat-signature-toggle-20260508/focused-build-log.txt`
- Saved derivation log: `/home/brittonr/.local/state/crunch/logs/lzaz8x66njc1m5r2xda8744s4w5yghwq-diag-gcc40-c-parse-boundary.drv.log`
