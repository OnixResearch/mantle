# Oracle Checkpoint: Cargo-free Fixed-point Proof

## Question

Can the external `/tmp` proof bundle and pueue transcript be accepted as sufficient evidence for the bounded Mantle Cargo-free stage1→stage2 fixed-point claim without committing multi-megabyte raw receipts?

## Inspected Evidence

- `scripts/prove-cargo-free-fixed-point.rs` records blocker details in `meta.json`, returns blocked `StageResult`s for post-command evidence failures, prepends a stage-local `cargo` shim to `PATH`, and sets `CARGO` to that shim.
- Synthetic blocker proof: `/tmp/mantle-fp-blocker-meta-final2.veosVY/meta.json` showed `status=blocked`, `stage1.success=false`, and a durable blocker for an empty receipt after the fake command exited 0.
- Synthetic ambient-Cargo proof: `/tmp/mantle-fp-cargo-meta-final2.o3Y1FO/meta.json` showed `status=blocked`, `stage1.cargo_marker_absent=false`, and blocker `cargo guard was invoked` after a fake Mantle binary ran `cargo --version` through ambient `PATH`.
- Source-digest stability guard: `src/rust_plan.rs` now skips `.agent/`, `.pi/`, `.git/`, `.jj/`, `target/`, and root `cairn/` while still hashing nested source files; focused test `path_source_digest_ignores_root_metadata_without_hiding_source_changes` passed.
- Full proof rerun: pueue task 62, bundle `/tmp/mantle-cargo-free-fixed-point-proof/run-final2-20260530T051100Z`.
- Full proof `meta.json`: `status=success`, `fixed_point=true`, stage1 units `599`, stage2 units `599`, failed units `0` for both stages, and `cargo_marker_absent=true` for both stages.
- Fixed-point digest: stage1 and stage2 Mantle binaries both `90c5ca1e91207c38d85131e7c988af61cb4477c0053f0042fae6a6989f95cf99`.
- Compact repo-local receipt summary: `evidence/fixed-point-receipt-summary.json` records matching stage unit IDs, source digests, rustc-args digests, environment digests, and output artifact digests.

## Decision

Accept the bounded Mantle Cargo-free fixed-point proof for this change. The raw receipts and binaries remain in `/tmp` because each full receipt is multi-megabyte, but the committed summary records the fields needed to audit the claim and the pueue log records the command result.

## Owner

Agent owns this checkpoint for the current proof hardening session. Human maintainer owns any future release-grade or Crunch bootstrap evidence claim.

## Next Action

Keep the bounded Mantle proof healthy in future native-planning changes. Do not extend this decision to full Crunch bootstrap, release reproducibility, or full Cargo-compatibility claims without separate evidence.
