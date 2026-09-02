# Validation summary (V5/V6, 2026-09-02)

All commands and exact output live beside this file.

## Oracle checkpoint

- **Question:** Does the source-review adapter bind policy-counted review approval to the exact released source and independently verified reviewer authority, without requiring review for generic releases?
- **Inspected evidence:** 261 `crunch-release-core` tests pass (26 new source-review core tests). 166 `release_cli` tests pass, including 16 new source-review CLI tests covering generic not-required reporting, verified-optional with an attachment, satisfied two-reviewer policy, stale source subject, stale claim root, policy mismatch, insufficient approvals, role confusion with a build-witness domain, invalid signature, excluded author, duplicate key under two labels, needs-revision, unknown reviewer key, tampered attachment identity, tampered bundled member digest, and absent reviewer authority failing closed. Strict first-party Clippy passes with `-D warnings`. Formatting and `git diff --check` pass. The machine-contract checker passes with the new contracted `release.source-review-attachment` surface (24 contracted, 57 classified). The exact Tiger Style Nix gate passes with no new allowances. Cairn validation reports no findings. Tracey reports 155/155. Proposal, design, and tasks gates return PASS.
- **Decision:** Accept. The pure core owns attachment structure, canonical identities, subject linkage, reviewer-key policy, role separation, and distinct counting. The shell owns file reads, bundle placement, and Ed25519 verification through the pinned Artifact Auth boundary. Generic manifests serialize identically without an attachment; presence requires operator reviewer authority and fails closed otherwise.
- **Owner:** Mantle maintainers.
- **Next action:** Commit, then run the bounded full Nix check with exact independent blockers preserved.

## Repair notes

- Initial evaluation logic exceeded Tiger Style function-length, assertion-density, nesting, parameter-shape, and overflow bounds; it was decomposed into focused phase helpers (`attachment_preflight_failure`, `subject_link_failure`, `role_domain_failure`, `select_distinct_approvals`, `build_signature_evidence`, `evaluate_authentications`, `finish_authentication`) with debug assertions and checked arithmetic, without any lint allowance.
- Optional-mode default threshold falls back to one distinct approval; a policy without reviewers is valid but unsatisfiable, so the shell requires explicit `--trusted-reviewer` keys whenever an attachment is present.
- The machine-contract generator exposed a pre-existing stale `build.build-json-report` producer binding inherited from `origin/main`; this change refreshed it through the generator without altering any schema semantics.

## Non-claims

Verification proves attributable review action over the exact source identity only. It does not prove source correctness, review completeness, reviewer competence, build correctness, reproducibility, witness independence, or release eligibility.
