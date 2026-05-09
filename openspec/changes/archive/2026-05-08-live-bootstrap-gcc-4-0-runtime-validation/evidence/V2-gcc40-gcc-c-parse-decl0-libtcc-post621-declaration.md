# V2 gcc40 c-parse decl0 libtcc post-621 declaration diagnostic

Task-ID: V2 focused TinyCC `libtcc.c` post-`tcc_open` declaration-shape diagnostic for the GCC 4.0 c-parse `decl0` predecessor compile boundary.
Covers: bootstrap.gcc40.runtime-validation

## Command

The normal Crunch diagnostic derivation was updated in `bootstrap/diag-gcc40-c-parse-boundary.ncl` with `libtcc_post621_declaration_*` probes. A direct local replay was also run from the restored prior Crunch outputs because fresh end-to-end diagnostic rebuilds currently spend the timeout rebuilding the predecessor chain:

```sh
bash openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-post621-declaration-20260509/run-local-probes.sh \
  > openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-post621-declaration-20260509/focused-local-probes-prefix621-cleaned.txt 2>&1
```

Replay inputs:

- TinyCC: `.crunch-drain/post621-restored-store/b557z7jmjqnlz118c8zvwdjvkv4v5qb3-tcc-0.9.27-musl-v2/bin/tcc`
- musl headers: `.crunch-drain/post621-restored-store/g38fpdz6wrs7n2p5f3afqbcb3w4s2m64-musl-1.1.24-tcc-musl/include`
- TinyCC source: `.crunch-drain/post621-restored-store/4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src`

## Focused results

All post-621 variants returned `rc=139` under both common and `ONE_SOURCE=1` flags:

| Variant | common | one_source |
| --- | ---: | ---: |
| `prefix_only` | 139 | 139 |
| `int_global` | 139 | 139 |
| `int_extern` | 139 | 139 |
| `typedef_int` | 139 | 139 |
| `static_func_prototype_void` | 139 | 139 |
| `static_func_prototype_tccstate` | 139 | 139 |
| `static_func_empty_void` | 139 | 139 |
| `struct_ptr_global` | 139 | 139 |

- Direct `diag-tcc-decl0-runtime:` markers emitted: 0
- Fresh Crunch diagnostic attempts: `run.log` records repeated dependency/cold-cache blockers and one 590s timeout before the focused local replay was used.

## Interpretation

The new diagnostic separates generic post-prefix declarations from the exact `tcc_compile` name/signature tested in the previous compile-signature slice. The replay shows the currently restored post-`tcc_open` prefix itself still trips the predecessor TinyCC (`rc=139`), so the next useful repair should move earlier than `tcc_compile` identity and target the source context accumulated by the line-621 prefix after the paired varargs cleanup work. No direct `decl0` runtime success is claimed.
