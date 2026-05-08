# V1 grep 2.4 runtime-validation rerun evidence

Task-ID: V1
Covers: bootstrap.part.grep.2.4.runtime-validation
Captured: 2026-05-08T21:42:24Z

## Result

A focused runtime-validation rerun for `bootstrap/grep-2.4-musl.ncl` was already captured during the archived binutils-tcc chain drain and is copied into this scoped follow-up so it can stand alone.

Source archive: `openspec/changes/archive/2026-05-05-live-bootstrap-binutils-tcc-chain-runtime-validation/evidence/`

Copied artifacts:

- `V3-grep-musl-success-validation-summary.json`
- `V3-grep-musl-success-validation-summary.md`
- `V3-grep-musl-success-build.stdout.log`
- `V3-grep-musl-success-build.stderr.log`
- `V1-grep-2.4-musl-root-derivation.log`

The validation summary reports `status: passed`, `doctor_ok: true`, `build_attempted: true`, and `build_exit_code: 0`.

Original command context from the validation summary:

- target: `bootstrap/grep-2.4-musl.ncl`
- store: `/home/brittonr/git/crunch/crunch/.crunch-drain/grep-musl-store9`
- state dir: `/home/brittonr/.local/state/crunch`
- store prefix: `/crunch/store`
