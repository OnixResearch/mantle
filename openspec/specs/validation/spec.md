# validation Specification

## Purpose
TBD - created by archiving change avoid-tmpfs-validation-failures. Update Purpose after archive.
## Requirements
### Requirement: Self-hosting proof helper uses disk-backed scratch by default

The self-hosting proof helper MUST resolve its scratch root from
`CRUNCH_PROOF_SCRATCH_DIR` when that environment variable is set.

Relative `CRUNCH_PROOF_SCRATCH_DIR` values MUST be anchored to the repo root.
Absolute `CRUNCH_PROOF_SCRATCH_DIR` values MUST be used as-is.

When `CRUNCH_PROOF_SCRATCH_DIR` is unset, the helper MUST use the repo-root
anchored repo-local path `target/self-hosting-proof/work/` instead of ambient
`/tmp` for its multi-GiB compiler artifacts and temporary files.

The selected scratch root MUST drive both `TMPDIR` and `CARGO_TARGET_DIR`
through subdirectories under that root.

A scratch root is usable only when the helper can create it if needed,
confirm it is a directory, and confirm it is writable.

#### Scenario: tmpfs `/tmp` does not become the default proof scratch root

- GIVEN a host where `/tmp` is tmpfs and can fill independently of the main filesystem
- AND `CRUNCH_PROOF_SCRATCH_DIR` is unset
- WHEN a contributor runs `scripts/prove-self-hosting.sh` from the repo root
- THEN the helper selects `target/self-hosting-proof/work/` instead of ambient `/tmp`
- AND the helper routes both `TMPDIR` and `CARGO_TARGET_DIR` under that selected root

#### Scenario: Default proof scratch selection is reported

- GIVEN `CRUNCH_PROOF_SCRATCH_DIR` is unset
- WHEN `scripts/prove-self-hosting.sh` starts
- THEN it reports the selected repo-local scratch path in its startup diagnostics
- AND it reports that the default repo-local policy was used

#### Scenario: Explicit proof scratch override is respected

- GIVEN a contributor sets `CRUNCH_PROOF_SCRATCH_DIR` to a disk-backed path
- WHEN `scripts/prove-self-hosting.sh` starts
- THEN it uses that operator-selected scratch root
- AND it reports that selected path in its startup diagnostics
- AND it reports that `CRUNCH_PROOF_SCRATCH_DIR` supplied the scratch root

#### Scenario: Relative proof scratch override is repo-root anchored

- GIVEN a contributor sets `CRUNCH_PROOF_SCRATCH_DIR` to a relative path
- AND they invoke `scripts/prove-self-hosting.sh` from outside the repo root cwd
- WHEN the helper starts
- THEN it resolves that relative scratch root against the repo root

#### Scenario: Absolute proof scratch override is preserved

- GIVEN a contributor sets `CRUNCH_PROOF_SCRATCH_DIR` to an absolute path
- WHEN the helper starts
- THEN it uses that absolute path as-is

#### Scenario: Explicit unusable proof scratch override fails with no fallback

- GIVEN a contributor sets `CRUNCH_PROOF_SCRATCH_DIR`
- AND that selected scratch path cannot be created, is not a directory, or is not writable
- WHEN `scripts/prove-self-hosting.sh` starts
- THEN it fails before proof work begins
- AND it does not fall back to `target/self-hosting-proof/work/`
- AND the error names `CRUNCH_PROOF_SCRATCH_DIR` as the scratch redirect interface

#### Scenario: No usable default proof scratch root fails before proof work

- GIVEN `CRUNCH_PROOF_SCRATCH_DIR` is unset
- AND the repo-local path `target/self-hosting-proof/work/` cannot be created or used
- WHEN `scripts/prove-self-hosting.sh` starts
- THEN it fails before proof work begins
- AND the error names `CRUNCH_PROOF_SCRATCH_DIR` as the scratch redirect interface

### Requirement: Proof scratch capacity preflight is explicit and early

`scripts/prove-self-hosting.sh` MUST fail before the long proof run begins when
the selected scratch filesystem has less than 4 GiB free.

The failure MUST name the selected scratch root and `CRUNCH_PROOF_SCRATCH_DIR`
as the override mechanism that can redirect it.

#### Scenario: Below-threshold proof scratch fails fast

- GIVEN the selected scratch filesystem has less than 4 GiB free
- WHEN the helper performs its preflight
- THEN it exits before the long compile or proof build begins
- AND the error names the selected scratch root
- AND the error tells the contributor to set `CRUNCH_PROOF_SCRATCH_DIR` to redirect scratch

### Requirement: Direct heavyweight cargo guidance is documented

The repo MUST document how contributors should set disk-backed `TMPDIR` and
`CARGO_TARGET_DIR` when they bypass checked-in helpers and run heavyweight
`cargo` validation commands directly.

That documentation MUST NOT replace `scripts/prove-self-hosting.sh` as the
canonical self-hosting proof entry point.

#### Scenario: Docs show direct cargo scratch guidance without replacing proof helper

- GIVEN a contributor wants to run a heavyweight direct `cargo` validation command
- WHEN they consult the repo docs
- THEN the docs show how to set `TMPDIR` and `CARGO_TARGET_DIR` to disk-backed paths
- AND the docs still direct self-hosting proof runs to `scripts/prove-self-hosting.sh`

