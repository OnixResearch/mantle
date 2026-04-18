# Tasks: classify dependency audit findings

## Phase 1: Spec and triage boundary

- [x] Add the `dependency-hygiene` delta spec covering the checked-in audit
      entry point, actionable-fix preference, and waiver metadata rules
- [x] Capture the current audit finding set and classify each finding by
      ownership and actionability before adding persistent waivers

## Phase 2: Remediation and policy

- [x] Upgrade or otherwise remediate findings that are directly controlled from
      this repo, including tractable manifest or lockfile updates
- [x] Add checked-in dependency-audit policy/config anchored on `cargo deny`
      in root `deny.toml`, including the AGPL-compatible license allow-list
- [x] Add narrow waiver records only for the findings that remain blocked after
      remediation, with advisory/class, affected crate, ownership path,
      rationale, and review trigger or expiry condition
- [x] Document in `docs/dependency-audit.md` how runtime, dev-only,
      vendored, and upstream-blocked findings are distinguished in the repo's
      audit policy

## Phase 3: Verification

- [x] Run the checked-in dependency-audit entry point and keep the transcript
- [x] Verify the final policy leaves no unclassified findings in the current
      audit set
- [x] Verify each remaining waiver targets a specific advisory or finding
      identifier plus the affected crate, not a blanket category suppression
- [x] Keep evidence for each direct fix that was chosen instead of a waiver
      where the repo controlled the upgrade path

## Validation

- [x] Run `openspec validate classify-dependency-audit-findings`
- [x] Run proposal, design, and tasks gates for
      `classify-dependency-audit-findings`
