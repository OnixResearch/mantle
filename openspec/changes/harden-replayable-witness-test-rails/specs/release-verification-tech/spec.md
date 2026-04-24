## ADDED Requirements

### Requirement: Replayable witness scratch validation MUST stay helper-compatible

Replayable witness scratch validation MUST accept only real helper-owned
`tmp/` and `cargo-target/` directories at scratch-root top level and MUST
reject symlinked or non-directory helper-owned entries before the rebuild
workflow starts.
ID: release.verification.tech.witness.rebuild.scratch

#### Scenario: Helper-owned scratch directories are accepted

- GIVEN a witness scratch root that already contains top-level `tmp/` and
  `cargo-target/` directories created by `./scripts/rebuild-witness-request.sh`
- WHEN `crunch release witness-rebuild` validates that scratch root
- THEN validation succeeds
- AND the rebuild workflow may continue

#### Scenario: Symlinked helper-owned scratch entry is rejected

- GIVEN a witness scratch root whose top-level `tmp/` or `cargo-target/` entry
  is a symlink
- WHEN `crunch release witness-rebuild` validates that scratch root
- THEN the command exits non-zero before the workflow driver starts
- AND the diagnostic names the symlinked helper-owned entry rejection

#### Scenario: Non-directory helper-owned scratch entry is rejected

- GIVEN a witness scratch root whose top-level `tmp/` or `cargo-target/` entry
  is a regular file or another non-directory node
- WHEN `crunch release witness-rebuild` validates that scratch root
- THEN the command exits non-zero before the workflow driver starts
- AND the diagnostic names the helper-owned entry type mismatch

### Requirement: Replayable witness no-launch assertions MUST be observable

Replayable witness regression tests MUST back any "workflow driver never
launched" claim with an observable launch seam that the fake witness-rebuild
 driver implementation actually consumes.
ID: release.verification.tech.witness.rebuild.testing

#### Scenario: Fake driver writes the consumed launch signal

- GIVEN a replayable witness test configures the fake witness-rebuild driver
  with a launch-signal path
- WHEN the fake driver process starts
- THEN the fake driver writes that launch signal before it reports success or
  failure

#### Scenario: Preflight rejection proves launch never happened

- GIVEN a replayable witness negative test that expects preflight rejection
  before the workflow driver starts
- WHEN the command exits non-zero
- THEN the test asserts the rejection diagnostic
- AND the test asserts the consumed launch signal is absent
