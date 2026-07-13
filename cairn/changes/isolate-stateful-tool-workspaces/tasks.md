## Implementation

- [x] [serial] I1 Inventory sandbox scratch roots, remote worker leases, action identity/admission, secret redaction, GC roots, and report seams; classify current state as ephemeral, immutable, or mutable. r[build_correctness.stateful_workspace_modes]
- [x] [depends:I1] I2 Add typed bounded `none`, `immutable-snapshot`, and `mutable-session` policies, compatibility facts, lease records, quota/retention rules, and claim classes. r[build_correctness.stateful_workspace_modes]
- [x] [depends:I2] I3 Implement pure mode admission, compatibility digest, lease transition, claim downgrade, quota/retention, scrub, snapshot, path-policy, and quarantine planning. r[build_correctness.stateful_workspace_modes]
- [x] [depends:I3] I4 Add shell-owned local workspace creation, exclusive locking, stable guest-path mounting, bounded scan/scrub, cleanup/quarantine, and immutable snapshot ingestion. r[build_correctness.mutable_workspace_claim_boundary]
- [x] [depends:I3] I5 Add worker/coordinator workspace registration and job/attempt/fence-bound leases; reject stale, concurrent, cross-worker, or cross-authority mutation. r[remote_builds.stateful_workspace_leases]
- [x] [depends:I4,I5] I6 Integrate workspace policy into sandbox requests and reports without adding host paths or mutable workspace identity to shared action-result candidates. r[build_correctness.mutable_workspace_claim_boundary]
- [x] [depends:I2] I7 Add typed Nickel defaults and bounds for modes, mount paths, compatibility fields, quotas, retention, scrub policy, snapshot policy, and clean-rebuild proof policy. r[remote_builds.stateful_workspace_leases]
- [x] [depends:I6,I7] I8 Add optional clean-rebuild comparison that can establish matching output-set evidence without relabeling the original mutable execution as hermetic. r[build_correctness.mutable_workspace_claim_boundary]

Evidence: `evidence/implementation-inventory.md` records seam ownership and trust boundaries. `evidence/validation-transcript.txt` proves the pure core, no-follow shell, CAS ingestion, sandbox transport, typed Nickel parity, remote restart/fencing, mutable cache boundary, reports, and clean comparison.

## Verification

- [x] [depends:I3] V1 Positive: equivalent immutable snapshot manifests produce stable object refs and action identity; compatible mutable lease facts produce deterministic accept/renew/release plans. r[build_correctness.stateful_workspace_modes]
- [x] [depends:I3] V2 Negative: reject unknown modes, invalid guest paths, path traversal, symlink escape, oversized quotas, arithmetic overflow, wrong toolchain/action compatibility, stale fences, concurrent owners, and cross-authority reuse. r[build_correctness.stateful_workspace_modes]
- [x] [depends:I4] V3 Positive: run a tool fixture twice in one mutable workspace, observe bounded warm-state reuse, preserve stable guest paths, and report the required evidence downgrade. r[build_correctness.mutable_workspace_claim_boundary]
- [x] [depends:I6] V4 Negative: prove mutable-session execution cannot publish or satisfy a strong shared action result and cannot leak secret fixtures or host paths into a snapshot. r[build_correctness.mutable_workspace_claim_boundary]
- [x] [depends:I5] V5 Positive and negative: restart with current leases, reject stale worker mutations, quarantine failed scrub/cleanup, and enforce retention without deleting active workspaces. r[remote_builds.stateful_workspace_leases]
- [x] [depends:I8] V6 Positive and negative: compare clean and warm rebuild outputs, admit matching comparison evidence, and report divergence without claiming cache correctness or mutating the original result. r[build_correctness.mutable_workspace_claim_boundary]
- [ ] [depends:I8] V7 Run focused sandbox/store/remote/report tests, Nickel contract tests, format and first-party lint checks, Cairn validate, Tracey coverage, and proposal/design/tasks gates with exact evidence. r[remote_builds.stateful_workspace_leases]

Evidence summary: focused tests passed with 2 snix-build, 23 crunch-build, 2 integration, 11 remote/Nickel, and report/schema checks; see `evidence/validation-transcript.txt`. V7 remains open only because the repository-wide strict Clippy gate stops in pre-existing unrelated files and repository-wide Tracey reports unrelated accepted-spec gaps. The focused strict Clippy run passes after naming only those pre-existing lint classes as waivers. Exact final gate commands and blocker output are appended to `evidence/validation-transcript.txt`.
