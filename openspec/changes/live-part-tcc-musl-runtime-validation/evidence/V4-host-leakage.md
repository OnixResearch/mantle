# V4 host-leakage scan: failed build attempt

- Task-ID: V4
- Covers: `bootstrap.part.tcc.musl.runtime-validation`
- Scope: validation-runner summary, captured build stdout/stderr, and saved root derivation log for this failed attempt.
- Runner leakage findings: `[]`
- Coarse builder-log findings: none reported by the validation runner.
- Expected runner metadata paths: `build.stdout.log` records explicit local evidence/store/state paths under `/home/brittonr/git/crunch/.../.crunch-drain/...`; these are validation-runner metadata, not sandbox builder leakage.

This scan is limited to the failed attempt. It does not prove the eventual successful builder transcript; rerun after the blocker is repaired.
