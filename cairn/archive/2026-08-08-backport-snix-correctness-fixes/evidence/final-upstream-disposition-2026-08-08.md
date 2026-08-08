# Final upstream disposition review

## Question

Does the final implementation match every selected behavior in the upstream ledger without importing deferred APIs or overstating upstream status?

## Inspected evidence

- Final branch diff from `origin/main`.
- `evidence/upstream-review.md`.
- Focused implementation evidence and exact post-change test output.

## Decision

| CL | Final disposition | Local evidence |
|---|---|---|
| 31145 | Adapted | Snix and Mantle requested-digest guards; zero-mutation negative test. |
| 31491 | Adapted | Complete multi-frame zstd decoding and malformed or over-limit rejection. |
| 31492 | Adapted | Directory-base URL normalization with subpath fixtures. |
| 31478 | Adapted; upstream remains open | Checked castore directory-size repair and regression tests. |
| 31496 | Adapted; upstream remains open | FUSE `DT_*` mapping and negative `S_IF*` tests. |
| 31495 | Adapted | Nonzero root and node link-count attributes. |
| 31306 | Adapted | Owned redb state and complete blocking transaction boundary. |
| 30571 | Adapted; upstream remains open | Near-only PathInfo listing and panic-on-list far fixture. |
| 31157 | Adapted | Default bounded `BufReader` with `copy_buf`; no fixed oversized capacity. |
| 31150 | Adapted for Mantle policy | One filter governs formatting, progress, and additional layers as required by the local spec. |

CL 31448 and CL 31272 remain deferred after the recorded trigger recheck. Other rejected or deferred changes remain outside this change.

## Owner

Mantle store and vendored-runtime maintainers own these local adaptations.

## Next action

Run the lifecycle gates, sync the accepted delta, archive the change, and rerun post-archive validation.

## Claim limit

This review proves bounded local adaptation and test coverage. It does not prove complete Snix parity, acceptance of open upstream changes, whole-store correctness, or release eligibility.
