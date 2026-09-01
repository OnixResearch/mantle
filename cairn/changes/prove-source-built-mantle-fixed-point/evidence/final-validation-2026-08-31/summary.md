# Final validation

## Proof evidence

V98 is the acceptance evidence for the source-built fixed point. Its external
bootstrap trust report returned `complete`, `fixed_point_verified: true`, and
`root_action_trust_complete: true`. It reported no blockers.

The proof evidence is under
`../v98-ptrace-request-abi-2026-08-31/`. The report was written outside the
proof root, so it did not change the authenticated proof-bundle digest.

## Required Rust checks

The requested `nix develop -c ...` commands did not enter the development
shell. Nix failed while fetching vendored crate derivations because the remote
crate mirror returned HTTP 403. The original command, status, and complete log
are retained for each attempt.

The documented local nightly, clang, mold, pkg-config, OpenSSL, and static
BusyBox environment then ran the checks directly.

Passed:

- `release_cli`: 149 passed, 0 failed;
- bootstrap trust-report CLI: 3 passed, 0 failed;
- root action trust: 2 passed, 0 failed;
- Rust action plan and shell: 16 passed, 0 failed;
- Rust-provider action policy: 3 passed, 0 failed;
- trust-report core and shell: 10 passed, 0 failed;
- ptrace supervision: 12 passed, 0 failed, with one test thread;
- machine-contract self-test: PASS;
- Rust formatting: PASS;
- `git diff --check`: PASS.

The proof-specific suites cover positive and negative action authority,
reconciliation, event bounds, generated executable promotion, digest drift,
raw-audit tampering, path traversal, incomplete adapter coverage, cache-only
completion, and ptrace deny-before-exec behavior.

## Preserved repository-wide blockers

The broad `bootstrap_parity_cli` suite reports 14 passed and five failed
existing parity fixtures:

- `bootstrap_parity_report_rejects_missing_final_runtime_member`;
- `bootstrap_parity_report_rejects_untrusted_acceptance_fingerprint`;
- `bootstrap_parity_report_rejects_stale_generated_artifact_status`;
- `bootstrap_parity_report_accepts_independently_receipted_early_native_rows`;
- `bootstrap_parity_report_accepts_independently_receipted_final_native_rows`.

The full machine-contract check passes its self-test, then rejects existing
root JSON producer-family inventory gaps. Its exact source list is in
`machine-contract-check-corrected.log`.

The broad root-package Clippy command reaches existing first-party warnings and
fails with warnings denied. The log includes dead-code findings, existing
large-enum and style findings, and newer-nightly Clippy findings. No warning
budget or source allowance was added.

Cairn validation and the proposal and design gates stop before repository
inspection because `cairn-policy/generated/cairn-policy.json` contains the
unsupported field `task_marker_policy.markers`. This is the preserved policy
compatibility blocker. The task gate records the same blocker after all task
evidence is complete.

These broad blockers do not alter V98's authenticated fixed-point receipt or
its independently validated complete root action trust result. They prevent an
honest archive claim until their owning repository work repairs them.
