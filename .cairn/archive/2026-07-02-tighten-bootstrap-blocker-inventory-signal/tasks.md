## Implementation

- [x] [serial] I1 Inspect the current report-only/enforcement exit path and split report-generation success from clean-baseline failure. r[bootstrap_inventory.blocker_signal]
  Evidence: pueue task 1594 showed report-only and enforcement both exited 1 before the fix; pueue task 1685 shows report-only exits 0 after valid report generation while enforcement still exits 1 on remaining actionable findings.
- [x] [serial] I2 Refine blocker classification so evidence metadata with explicit durable suppressions is rendered separately from actionable source findings. r[bootstrap_inventory.blocker_signal]
  Evidence: JSON schema version 2 now reports 38 actionable findings, 306 metadata suppressions, and 90 source suppressions; Markdown renders metadata suppressions, source suppressions, promotion claims, and actionable findings separately.
- [x] [serial] I3 Update usage/help text and Markdown/JSON summaries to name report-only and enforcement semantics precisely. r[bootstrap_inventory.blocker_signal]
  Evidence: wrapper help and `bootstrap/BLOCKER-INVENTORY.md` now describe report-only success vs clean-baseline enforcement failure; report headers expose the split counts.

## Verification

- [x] [serial] V1 Add positive coverage proving report-only mode exits successfully while writing valid JSON/Markdown when known blockers remain. r[bootstrap_inventory.blocker_signal]
  Evidence: `./scripts/check-bootstrap-blocker-inventory.sh --self-test --report-only` in pueue task 1685 exited 0 with 38 findings remaining; Steel parsed the generated JSON successfully.
- [x] [serial] V2 Add negative coverage proving enforcement mode fails when unsuppressed blockers or promotion claims are present. r[bootstrap_inventory.blocker_signal]
  Evidence: default enforcement in pueue task 1685 exited 1 with `FAIL: bootstrap blocker inventory is not clean`; self-test also covers promotion-only and promotion-plus-finding rejection.
- [x] [serial] V3 Add a metadata-suppression regression proving checked evidence files do not dominate actionable findings while an unsuppressed source blocker still remains counted. r[bootstrap_inventory.blocker_signal]
  Evidence: `run_self_tests()` now asserts metadata/source suppression classification, and the generated report keeps 306 metadata suppressions separate from 38 actionable findings.
- [x] [serial] V4 Run `./scripts/check-bootstrap-blocker-inventory.sh --self-test --report-only`, default enforcement check, `git diff --check`, and Cairn validation/gates; record evidence before archive. r[bootstrap_inventory.blocker_signal]
  Evidence: pueue task 1685 covers report-only, enforcement, shell syntax, and `git diff --check`; see `evidence/blocker-inventory-signal.md` for the transcript. Cairn validation/gates are recorded before archive.
