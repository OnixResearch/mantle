# Design

The v6 receipt remains nested under the existing build-frontier receipt and preserves `status=frontier-only`. It records a bounded diagnostic/source-frontier reduction only.

Validation must require:

- schema `mantle-gcc40-native-cc1-source-frontier-reduction-v6`;
- observed frontier fragments for later generated-header successes (`machmode` 120, `tree` 80/120/160/220, builtin empty/complex) and the real `c-parse.o` make failure;
- exact diagnostic-source markers from `bootstrap/diag-gcc40-c-parse-boundary.ncl`;
- stale v3/v4/v5 frontier wording rejection;
- no parity completion claim.
