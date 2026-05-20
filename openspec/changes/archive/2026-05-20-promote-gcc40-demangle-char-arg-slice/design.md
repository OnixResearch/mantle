## Context

The current demangle receipt `mantle-gcc40-native-demangle-slice-v3` verifies bounded flat/two-component/deep Itanium names and selected single-`int` arguments. The implementation intentionally remains a small static-buffer parser because prior malloc/free paths were fragile under the TinyCC/Mes runtime.

## Decision

Promote exactly one additional builtin type code, Itanium `c`, for selected flat/nested/deep function shapes, with the selected frontier recorded as the three-component nested char argument case.

## Non-Goals

- No arbitrary parameter lists.
- No double/float/pointer/reference/operator-name decoding.
- No full `cp-demangle` or native GCC 4.0 correctness claim.

## Risks / Mitigations

- Archive merge drift can drop prior scenarios. Inspect and repair the canonical bootstrap spec after archive.
- Stale markers can make old evidence look current. Add v3 markers to the forbidden stale-marker set and require v4 markers in the derivation.
