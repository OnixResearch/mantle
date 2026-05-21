# Design: GCC 4.0 c-parse make-log summary frontier

## Contract

Schema `mantle-gcc40-native-cc1-source-frontier-reduction-v9` remains frontier-only. The diagnostic derivation records compact, source-bound markers after the focused make attempt:

- `diag-cparse-frontier: cparse_make_cparse_o_compile_command=present`
- `diag-cparse-frontier: cparse_make_cparse_o_log_lines=...`
- `diag-cparse-frontier: cparse_make_cparse_o_log_bytes=...`

The marker names are checked in the receipt; dynamic values are observed in build logs but are not required as exact source markers.

## Non-goals

- No promotion of `gcc.4.0` beyond evidence-backed `partial`.
- No full native GCC 4.0 source-build or compiler correctness claim.
- No broad diagnostic matrix reintroduction.
