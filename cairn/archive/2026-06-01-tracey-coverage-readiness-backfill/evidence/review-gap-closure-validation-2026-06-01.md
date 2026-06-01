# Review gap closure validation

Task-ID: review-gap-closure-validation
Covers: verification_evidence.tracey_coverage_readiness

$ git diff --check
exit_status=0

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
  "input_hash": "1f99a1f27581ed64cfaeacd3c6d0c85acf98ce0c29c695b3183fd4a320224734",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "760c14ab99db995bd719e50f56abab339742c76bd943e2bce6d8d306aa3bb3e1",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
exit_status=0
