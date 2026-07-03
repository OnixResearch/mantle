## Implementation

- [x] [serial] I1 Add a lifecycle evidence runner with an explicit per-change command list and dry-run mode. r[verification_evidence.cairn_lifecycle_runner]
- [x] [serial] I2 Extract pure lifecycle decision logic for task completion, command transcript presence, Cairn gate JSON, accepted-spec ID presence, archive path, and status summaries. r[verification_evidence.lifecycle_runner_fail_closed]
- [x] [serial] I3 Implement evidence transcript capture and bounded summary rendering for focused tests, format checks, Cairn gates, sync/archive, post-archive validation, and status checks. r[verification_evidence.lifecycle_evidence_transcript]
- [x] [serial] I4 Document the runner workflow and how it preserves proof-before-claim evidence. r[verification_evidence.cairn_lifecycle_runner]

## Verification

- [x] [serial] V1 Positive: run the runner against a fixture change and prove it captures command output, gates, sync/archive, post-archive validation, and final status evidence. r[verification_evidence.lifecycle_evidence_transcript]
- [x] [serial] V2 Negative: fail closed on unchecked tasks, missing evidence, failed Cairn gate, skeleton accepted-spec sync, dirty unexpected status, and command transcript absence. r[verification_evidence.lifecycle_runner_fail_closed]
- [x] [serial] V3 Run runner unit tests, a fixture dry-run, `cargo fmt -p mantle --check`, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[verification_evidence.cairn_lifecycle_runner]
