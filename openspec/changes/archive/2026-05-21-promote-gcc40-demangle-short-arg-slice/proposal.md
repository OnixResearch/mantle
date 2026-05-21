# Promote GCC 4.0 demangle short-arg slice

## Why

`gcc.4.0` remains partial because native/full demangler correctness is not proven. The current bounded demangle slice is a low-risk semantic seam with durable receipt validation. Adding exactly one adjacent Itanium type code improves real semantic coverage without claiming full cp-demangle correctness.

## What Changes

- Extend the bounded GCC 4.0 libiberty demangle semantic slice from zero-arg, single-int, single-char, and single-long Itanium function names to include one additional checked single-`short` argument shape.

## Scope

- Accept `_ZN3foo3bar3bazEs -> foo::bar::baz(short)` and preserve existing flat/two-component/nested regressions.
- Update `bootstrap/evidence/gcc-4.0-native-demangle-slice.json` to a v6 schema with fail-closed stale-marker checks.
- Update the bounded demangler implementation in `bootstrap/gcc-4.0.ncl` and parity validation/tests/spec wording.
- Keep `gcc.4.0` evidence-backed partial/non-promoted.

## Non-goals

- Full native GCC 4.0 demangler correctness.
- Broader argument lists, templates, operators, constructors/destructors, namespaces beyond the existing bounded depth, or source-build promotion.
