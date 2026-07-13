## Implementation

- [ ] [serial] [depends:persist-remote-attempt-observability] I1 Inventory route, assignment, attempt/fence, input, sandbox, immutable-log, transfer, workspace, admission, cleanup, and report facts available at each remote failure phase. r[operator_diagnostics.remote_failure_debug_bundle]
- [ ] [depends:I1] I2 Add bounded `mantle-remote-failure-debug-bundle-v1`, captured-artifact manifest, inspect summary, replay plan, and replay-comparison DTOs with domain-separated BLAKE3 refs. r[operator_diagnostics.remote_failure_debug_bundle]
- [ ] [depends:I2] I3 Implement pure bundle eligibility/canonicalization, capture allowlist and quota planning, sensitivity/redaction, inspect, replay-plan, comparison, retention, and stable reason-code kernels. r[operator_diagnostics.remote_failure_debug_bundle]
- [ ] [depends:I3] I4 Add shell-owned metadata-only bundle persistence over immutable refs and consume immutable attempt-log refs without introducing another log store. r[operator_diagnostics.remote_failure_debug_bundle]
- [ ] [depends:I3] I5 Add explicit pre-cleanup sandbox artifact capture with allowlisted relative paths/classes, bounded regular-file handling, CAS ingestion, cleanup/quarantine ordering, and retention roots. r[remote_builds.failure_debug_capture]
- [ ] [depends:I4] I6 Add operator inspect and replay-plan commands that validate refs, remain side-effect free, and render bounded redacted human/JSON summaries. r[operator_diagnostics.remote_failure_replay]
- [ ] [depends:I5,I6] I7 Add explicit replay execution as a new local/remote attempt through ordinary route, sandbox, transfer, output-admission, and action-result policy. r[operator_diagnostics.remote_failure_replay]
- [ ] [depends:I2] I8 Add typed Nickel debug capture, sensitivity, size/count/depth, retention, and replay policy with disabled-by-default payload capture. r[remote_builds.failure_debug_capture]
- [ ] [depends:I7,I8] I9 Add bundle refs, capture outcome, replay identity, comparison result, cleanup status, and non-claims to remote reports/status without exposing captured content. r[operator_diagnostics.remote_failure_debug_bundle]

## Verification

- [ ] [depends:I3] V1 Positive: permutation-equivalent bundle facts canonicalize identically; metadata-only bundles validate and inspect without worker access. r[operator_diagnostics.remote_failure_debug_bundle]
- [ ] [depends:I3] V2 Negative: reject malformed refs, wrong action/attempt/fence, oversized metadata, control characters, host paths, traversal, escaping symlinks, sockets/devices, secrets, unsupported file kinds, and arithmetic overflow before publication. r[operator_diagnostics.remote_failure_debug_bundle]
- [ ] [depends:I5] V3 Positive and negative: fail a multi-process remote fixture, capture one allowlisted bounded artifact before cleanup, reject disallowed/oversized artifacts, and prove capture failure cannot rewrite execution/output truth. r[remote_builds.failure_debug_capture]
- [ ] [depends:I6] V4 Inspect the bundle from a clean process with no original sandbox; prove inspect/replay-plan perform no execution, network, or stale-authority mutation. r[operator_diagnostics.remote_failure_replay]
- [ ] [depends:I7] V5 Replay under a new attempt/fence, compare matching and divergent phase/output evidence, and prove replay success or failure cannot rewrite the original result or bypass output admission. r[operator_diagnostics.remote_failure_replay]
- [ ] [depends:I9] V6 Exercise retention/GC with active replay leases, expired bundles, quarantined cleanup, missing immutable logs, and redaction assertions. r[remote_builds.failure_debug_capture]
- [ ] [depends:I9] V7 Run focused remote/log/sandbox/store/report tests, Nickel contract tests, format and first-party lint checks, Cairn validate, Tracey coverage, and proposal/design/tasks gates with exact evidence. r[operator_diagnostics.remote_failure_debug_bundle]
