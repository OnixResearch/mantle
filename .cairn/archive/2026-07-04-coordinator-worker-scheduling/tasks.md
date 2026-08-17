## Implementation

- [x] [serial] I1 Add pure coordinator state transitions for worker registration, capability matching, normalized-key dedupe, queue admission, leases, and resume summaries. r[remote_builds.coordinator_worker_runtime]
- [x] [serial] I2 Implement worker-dialed registration and coordinator dispatch shells with durable redacted state. r[remote_builds.coordinator_worker_runtime]
- [x] [serial] I3 Enforce resource limits, upload feasibility, logical store-prefix match, and output-trust preflight before assignment. r[remote_builds.coordinator_worker_runtime]
- [x] [serial] I4 Preserve resumable job/log/result state across restart when normalized build key and trust policy still match. r[remote_builds.coordinator_worker_runtime]

## Verification

- [x] [serial] V1 Positive: compatible worker registration receives a matching concrete job and duplicate client requests attach to one in-flight job. r[remote_builds.coordinator_worker_runtime]
- [x] [serial] V2 Negative: capability mismatch, output-trust blocker, conflicting live output claim, exhausted resource limit, store-prefix mismatch, and stale resume state fail closed. r[remote_builds.coordinator_worker_runtime]
- [x] [serial] V3 Run focused coordinator state/CLI tests plus Cairn validate and proposal/design/tasks gates for this change. r[remote_builds.coordinator_worker_runtime]
