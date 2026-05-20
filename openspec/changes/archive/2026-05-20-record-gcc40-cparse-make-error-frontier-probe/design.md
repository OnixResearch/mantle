# Design

The diagnostic derivation already reaches the real `make ... c-parse.o` target after v6 probes. Replace the direct failing make invocation with a capture block that:

1. Runs the same command with output redirected to a bounded log file.
2. Emits stable `diag-cparse-frontier:` markers for `cparse_make_cparse_o_rc`, output head/tail, and the preserved non-success boundary.
3. Exits with the captured nonzero status so the diagnostic derivation remains a failing diagnostic artifact, not a promotion.

The parity receipt remains nested under `source_frontier_reduction`, keeps `status=frontier-only`, and requires exact source markers from `bootstrap/diag-gcc40-c-parse-boundary.ncl`.
