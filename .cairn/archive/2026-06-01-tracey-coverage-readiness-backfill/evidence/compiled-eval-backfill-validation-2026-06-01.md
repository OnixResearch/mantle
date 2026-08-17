# Compiled-eval backfill validation

Task-ID: compiled-eval-backfill-validation
Covers: verification_evidence.tracey_coverage_readiness

This transcript was created after adding compiled-eval bridge refs, formatting `tools/tracey_refs.rs`, and updating the active Tracey tasks. No task file edits are made after these commands in this follow-up.

$ env PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/home/brittonr/.pi/agent/bin:/nix/store/4fi2lyz8xn4mhq1gk5sp3zgfw43fjl56-fd-10.4.2/bin:/nix/store/abq2d8kfixpmgn0sm843pf6jhv5s4qhg-ripgrep-15.1.0/bin:/home/brittonr/.local/bin:/home/brittonr/.nix-profile/bin:/run/wrappers/bin:/etc/profiles/per-user/brittonr/bin:/run/current-system/sw/bin rustfmt --check tools/tracey_refs.rs
exit_status=0

$ /nix/store/1bmgbnvw7cbwys5lwab9r8jlqpwpn473-cairn-0.1.0/bin/cairn tracey coverage --root . --json > cairn/changes/tracey-coverage-readiness-backfill/evidence/tracey-coverage-after-compiled-eval-batch-validation-2026-06-01.json
error: tracey coverage failed
exit_status=1

Coverage summary:

```json
{
  "valid": false,
  "requirements": 202,
  "referenced": 25,
  "missing_count": 177,
  "dangling_count": 0,
  "compiled_eval_missing": 0,
  "missing_by_group": [
    {
      "group": "rust_package_planning",
      "count": 176
    },
    {
      "group": "verification_evidence",
      "count": 1
    }
  ]
}

```

$ /nix/store/1bmgbnvw7cbwys5lwab9r8jlqpwpn473-cairn-0.1.0/bin/cairn validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 5,
  "valid": true
}
exit_status=0

$ /nix/store/1bmgbnvw7cbwys5lwab9r8jlqpwpn473-cairn-0.1.0/bin/cairn gate tasks tracey-coverage-readiness-backfill --root .
{
  "change": "tracey-coverage-readiness-backfill",
  "input_hash": "6f6276bb4dbd689b886c740e26415e7c4239675c40784d2b348c9d73613dffd3",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "aba8038a730bf4c4c2c65352b40e497117a4fa326a39491e19772bc09ac604ca",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
exit_status=0
