# V4 host-leakage scan: failed prerequisite attempt

- Task-ID: V4
- Covers: `bootstrap.part.bzip2.1.0.8.musl.evidence-isolated`
- Scope: validation-runner summary, captured build stdout/stderr, and saved root derivation log for the failed `bzip2-1.0.8-musl` attempt.
- Runner leakage findings: `[]`
- Coarse builder-log findings: none in `build.stderr.log` or `V2-bzip2-musl-root-derivation.log`.
- Expected runner metadata paths: `build.stdout.log` records the explicit local evidence/store/state paths (`/home/brittonr/git/crunch/.../.crunch-drain/...`) as validation-runner metadata, not sandbox builder leakage.

The scan is limited to the failed prerequisite attempt. It does not prove the eventual successful `bzip2-1.0.8-musl` builder transcript; that must be rescanned after `tcc-0.9.27-musl-v2` is repaired and this target builds.
