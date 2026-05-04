# V2-V4 patch 2.5.9 runtime validation

Task-IDs: V2, V3, V4
Covers: `bootstrap.part.patch.2.5.9.runtime-validation`

## Result

PASS: `bootstrap/patch-tcc.ncl` builds successfully under Crunch bootstrap validation, produces a `patch-2.5.9-tcc` output, and runs the in-builder smoke that prints `patch 2.5.9`, applies a simple unified diff to `smoke.txt`, and verifies the result with `grep '^new$' smoke.txt`.

## Output

- Logical output: `/crunch/store/zw9ma0lcl4kv9cacn1ah9avkgkf99nry-patch-2.5.9-tcc`
- Physical output: `.crunch-drain/patch-tcc-store/zw9ma0lcl4kv9cacn1ah9avkgkf99nry-patch-2.5.9-tcc`
- Derivation: `/crunch/store/iwzf16hhy966j56qyif2payncy5r8vim-patch-2.5.9-tcc.drv`
- Artifact attestation: `.crunch-drain/patch-tcc-state/attestations/artifacts/f27429badaf588f3a78c1107366bb21cc7d583ebab984e09817d59baa8baa905.json`

## Evidence

- `validation-summary.json` / `validation-summary.md`: validation wrapper result (`status: passed`, `build_exit_code: 0`).
- `build.stdout.log`: Crunch build report with `succeeded_total: 1`, `failed_total: 0`, output path, and attestation path.
- `patch-tcc.drv.log`: derivation transcript showing `patch --version`, `patching file smoke.txt`, and `new` from the smoke grep.
- `doctor.json`: preflight/doctor result for the local store/state/bwrap environment.

## Leakage scan

The validation wrapper reported no coarse host-path leakage findings in captured build output (`leakage_findings: []`) and no practical hermeticity audit events in the build report (`hermeticity_audit_events: []`).
