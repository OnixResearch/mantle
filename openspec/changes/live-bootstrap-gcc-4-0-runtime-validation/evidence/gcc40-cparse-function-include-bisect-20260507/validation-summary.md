# Bootstrap validation summary

- Schema: `crunch-bootstrap-validation-v1`
- Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
- Status: `inconclusive-timeout-before-target`
- Doctor OK: `true`
- Plan exit code: `0`
- Build attempted: `true`
- Build exit code: `None` (killed after timeout before target log emission)
- Store: `/home/brittonr/git/crunch/crunch/.crunch-drain/gcc40-cparse-function-include-bisect-store`
- State dir: `/home/brittonr/git/crunch/crunch/.crunch-drain/gcc40-cparse-function-include-bisect-state`

## Evidence

- Doctor JSON: `/home/brittonr/git/crunch/crunch/openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-function-include-bisect-20260507/doctor.json`
- Plan stdout: `/home/brittonr/git/crunch/crunch/openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-function-include-bisect-20260507/plan.stdout.log`
- Plan stderr: `/home/brittonr/git/crunch/crunch/openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-function-include-bisect-20260507/plan.stderr.log`
- Build stdout: `/home/brittonr/git/crunch/crunch/openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-function-include-bisect-20260507/build.stdout.log`
- Build stderr: `/home/brittonr/git/crunch/crunch/openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-function-include-bisect-20260507/build.stderr.log`

## Result

The function-after-include diagnostic extension evaluates and plans successfully. A fresh `bootstrap validate` attempt reached the Mes dependency build and was stopped after roughly 19 minutes with no target stdout/stderr and no `diag-gcc40-c-parse-boundary` derivation log, so this bundle does not claim a new runtime boundary. It records the scoped probe matrix and the plan proof needed for the next cached/runtime run.

## Host leakage scan

No coarse host-path needles were found in captured build output; target output was empty because the run was stopped before target execution.
