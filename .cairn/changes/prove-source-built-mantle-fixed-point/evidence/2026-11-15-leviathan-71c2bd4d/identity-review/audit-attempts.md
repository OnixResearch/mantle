# Audit attempt record

The first two remote reviewer-script executions were explicitly cancelled before a verdict or result file was emitted; neither is evidence of a cohort defect.

1. `identity-audit.log`: cancelled after static review found the profile scanner could append a non-target record's enormous one-line `content_hex` payload to its prefix buffer. The final script caps retained prefixes at 64 KiB.
2. `identity-audit-run2.log`: cancelled after a further completeness review found that physical role chunks were byte-compared but not independently checked against each record chunk BLAKE3, and the AWS perlasm source-tree consumer was not directly linked back to the original-source producer. The final script adds both checks, plus independent canonical plan/report pins and explicit selected-probe coverage.
3. `identity-audit-final.log`: also cancelled before a result was retrieved. The subsequent `patch-identity-v3.py` did not apply because its occurrence-count assertion failed. No physical criterion or verdict was accepted from these attempts.

All reviewer executions were restricted to the unique inert `/dev/shm/mantle-identity-review-71c2bd4d-gpt6luna-20261115` scratch on the proof host. No product, helper, generator, provider, controller, recipe, build, test, or proof was executed.

Parent continuation repairs the actual inspector rather than accepting the incomplete review: canonical record-body fingerprints use BLAKE3, complete Ring source routing and archive/file-kind checks are explicit, pretty-JSON record delimiters are decoded correctly, and Git blob comparison drains stdout while supplying all paths. The earlier manual stdin-write/undrained-stdout pattern could deadlock when Git filled its output pipe; this is an inspector risk, not evidence of a product defect.

The corrected parent execution uses a separately named remote `identity-audit-parent-v4.py` and `identity-review-parent-v4.json`. Only a completed actual result may discharge its physical criteria. The original product/frozen/cohort inputs remain read-only.
