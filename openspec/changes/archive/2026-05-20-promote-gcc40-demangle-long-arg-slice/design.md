## Context

The bounded demangle shim uses a small parser and static output buffer to avoid TinyCC/Mes runtime allocation pitfalls. The char slice is current v4 evidence.

## Decision

Promote exactly one new Itanium builtin type code: `l -> long`, using schema `mantle-gcc40-native-demangle-slice-v5` and markers `gcc40_cplus_demangle_long_arg_itanium_v5_boundary` / `gcc40_cp_demangle_long_arg_itanium_v5_boundary`.

## Non-Goals

No arbitrary type grammar, no pointer/reference/const support, no `cp-demangle` parity claim, and no native GCC 4.0 completion claim.

## Risks

OpenSpec archive may drop prior cumulative GCC version ladder scenarios; inspect and repair canonical spec drift after archive.
