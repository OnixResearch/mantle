## Why

The GCC 4.0 native libiberty demangle frontier now has checked zero-argument and single-`int` Itanium function slices. A selected single-`char` argument shape is the next small semantic expansion that improves native demangle coverage without implying arbitrary type decoding or full `cp-demangle` correctness.

## What Changes

- Promote one bounded single-`char` Itanium demangle shape, `_ZN3foo3bar3bazEc -> foo::bar::baz(char)`.
- Preserve flat, nested, deep zero-argument, and single-`int` regressions.
- Update native boundary and demangle receipts plus fail-closed stale-marker tests.

## Impact

- Files: `bootstrap/gcc-4.0.ncl`, GCC 4.0 evidence receipts, parity tests, and bootstrap OpenSpec scenarios.
- Non-goals: arbitrary type decoding, full native `cp-demangle`, full GCC 4.0 correctness, or live-bootstrap/Guix parity completion.
- Verification: focused GCC 4.0 parity tests, parity snapshot checker, strict OpenSpec validation, and whitespace diff checks.
