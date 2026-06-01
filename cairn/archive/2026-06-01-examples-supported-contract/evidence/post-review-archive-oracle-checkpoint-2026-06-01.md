# Post-review archive oracle checkpoint

Question: Can the `examples-supported-contract` archive claim be reviewed when the follow-up review range starts after the archive commit?

Inspected evidence: `cairn/archive/2026-06-01-examples-supported-contract/evidence/archive-validation-2026-06-01.md` records implementation commit `e648e267`, task gate `"verdict": "PASS"`, sync mutation, archive execution, post-archive path listing, post-archive `cairn validate --root .` with `"valid": true`, and Tracey summary with `examples_missing=`. `cairn/archive/2026-06-01-examples-supported-contract/evidence/implementation-validation-2026-06-01.md` records `cargo test -p mantle --test examples_inventory -- --nocapture` with `test result: ok. 7 passed; 0 failed; 0 ignored`.

Decision: Treat the archive claim as locally supported by archived evidence, but expose this checkpoint in the follow-up diff so same-family review can evaluate the claim without relying on excluded commits or paths.

Owner: Mantle maintainer / draining agent.

Next action: Keep future final summaries limited to claims whose evidence paths are included in the review range, or add a fresh checkpoint like this when summarizing work completed before the supplied review range.
