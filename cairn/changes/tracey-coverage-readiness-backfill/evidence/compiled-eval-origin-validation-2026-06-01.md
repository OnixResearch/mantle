# Compiled-eval origin checkpoint validation

Task-ID: compiled-eval-origin-validation
Covers: verification_evidence.tracey_coverage_readiness

$ bash -lc jq\ -r\ \'\ .groups\[\]\ \|\ select\(.group==\"compiled-eval\"\)\ \|\ .missing_ids\[\]\ \'\ cairn/changes/tracey-coverage-readiness-backfill/evidence/tracey-coverage-after-first-batch-grouped-2026-06-01.json\ \|\ sort\ \>\ /tmp/mantle-compiled-eval-missing.ids\ \&\&\ rg\ -o\ \'r\\\[compiled-eval\[\^\]\]+\\\]\'\ openspec/specs/compiled-eval-backends/spec.md\ \|\ sed\ \'s/\^.\*r\\\[//\;\ s/\\\]\$//\'\ \|\ sort\ \>\ /tmp/mantle-compiled-eval-spec.ids\ \&\&\ test\ \"\$\(wc\ -l\ \<\ /tmp/mantle-compiled-eval-missing.ids\)\"\ =\ \"21\"\ \&\&\ test\ \"\$\(wc\ -l\ \<\ /tmp/mantle-compiled-eval-spec.ids\)\"\ =\ \"21\"\ \&\&\ test\ -z\ \"\$\(comm\ -3\ /tmp/mantle-compiled-eval-missing.ids\ /tmp/mantle-compiled-eval-spec.ids\)\"\ \&\&\ \!\ rg\ -q\ \'compiled-eval\'\ cairn/specs
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
