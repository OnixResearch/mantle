## Why

GCC 4.0 native demangle evidence has advanced through bounded zero-argument, single-`int`, and single-`char` Itanium function-name slices. The next small frontier is one additional scalar builtin type code, `l`, for `long`, while preserving all prior regressions and bounded non-claims.

## What Changes

- Promote `_ZN3foo3bar3bazEl -> foo::bar::baz(long)` as the selected bounded shape.
- Preserve flat/two-component/three-component zero-arg, `int`, and `char` regressions.
- Update evidence receipts, parity checks, stale-marker denial, and OpenSpec bootstrap spec.

## Out of Scope

- Arbitrary type decoding.
- Operators, templates, substitutions, qualifiers, pointers/references, overload sets, or full `cp-demangle`.
- Full native GCC 4.0 correctness or live-bootstrap/Guix/StageX completion.

## Verification

Run focused GCC 4.0 parity tests, parity snapshot rail, strict OpenSpec validation, and whitespace checks.
