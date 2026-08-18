# Protected mini Stage0 diagnostic — 2026-07-27

The source-built mini Stage0 chain completed with exact outputs and zero
fallback. Audit validation then failed because the first command counter also
counted indented continuation arguments. It expected 54 Stage0 events but
observed 43.

The retained plan, inventory, generated recipe, and stderr log are diagnostic
only. The counter was corrected to count only command lines in column zero.
The later v6 proof supersedes this run.
