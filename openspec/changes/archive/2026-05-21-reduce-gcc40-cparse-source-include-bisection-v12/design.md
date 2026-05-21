# Design

Keep the active diagnostic compact. Before the focused `make c-parse.o` reproduction, run a small cumulative include-prefix probe over `c-parse.c`'s source-level include order rather than scraping the truncated make log again.

The diagnostic should record exact compact markers for the chosen prefix boundary, for example the ordered prefix and the first failing/suspect include. The receipt and parity validation must require those markers and continue to require the v11 make-log truncation evidence.

The slice remains diagnostic-only: it can narrow the source include frontier, but it must not change `gcc.4.0` parity status or claim native compiler/source-build correctness.
