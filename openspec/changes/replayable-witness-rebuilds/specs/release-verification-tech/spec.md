## ADDED Requirements

### Requirement: CLI MUST rebuild witness requests into witness material

Crunch MUST provide `crunch release witness-rebuild <request-dir>` to drive the
witness-side rebuild from an exported request directory and create witness
material without hand-wiring the rebuild recipe. The repo MUST also provide
`./scripts/rebuild-witness-request.sh` as the checked-in preflight wrapper for
that command.
ID: release.verification.tech.witness.rebuild.cli

The command MUST:
- verify the request-directory schema, layout, and bundled release-evidence
  bundle before rebuild,
- match the recorded workflow identity as the exact pair
  `(workflow_command="./scripts/prove-self-hosting.sh",
  workflow_version="crunch-self-hosting-proof-v2")` for the first supported
  rebuild path,
- accept the witness metadata already required by `crunch attest
  witness-create`, including signing-key selection, witness identity, system,
  toolchain, and host class,
- reject unsupported request layouts or unsupported recorded workflow
  identities before rebuild starts,
- let `./scripts/rebuild-witness-request.sh` discover `bwrap`, resolve and
  absolutize a real static `SNIX_BUILD_SANDBOX_SHELL`, derive a controlled
  scratch root, and rewrite `TMPDIR` and `CARGO_TARGET_DIR` under that root
  before invoking the command,
- rebuild the published outputs using the request's recorded workflow identity
  instead of an ad hoc operator-chosen recipe,
- reject rebuilt output count or output-name mismatches before writing witness
  sidecars, and
- write witness sidecars plus a rebuild audit directory to deterministic paths
  that `crunch attest witness-import` can consume, and
- keep any helper `--check` mode strictly preflight-only so successful preflight
  does not count as proof of a successful rebuild.

#### Scenario: Valid request rebuild creates witness sidecars and audit bundle

- GIVEN an exported witness request directory with a supported layout and a
  verified bundled release-evidence bundle
- WHEN the witness operator runs `crunch release witness-rebuild` with valid
  witness metadata
- THEN the command rebuilds the published outputs through the recorded
  workflow identity
- AND it writes witness sidecars for the rebuilt outputs
- AND it writes a deterministic rebuild audit directory alongside those
  sidecars

#### Scenario: Unsupported request workflow fails before rebuild

- GIVEN an exported witness request directory whose recorded workflow identity
  or request layout is unsupported by the installed crunch version
- WHEN the witness operator runs `crunch release witness-rebuild`
- THEN the command exits non-zero before the rebuild starts
- AND it names the unsupported workflow identity or request layout

#### Scenario: Helper preflight does not count as rebuild proof

- GIVEN the witness host runs `./scripts/rebuild-witness-request.sh --check`
- WHEN preflight succeeds
- THEN the helper reports only prerequisite status
- AND it does not claim to have produced publishable witness sidecars

#### Scenario: Rebuilt output mismatch blocks witness publication

- GIVEN an exported witness request directory whose published output list does
  not match the locally rebuilt output count or output names
- WHEN `crunch release witness-rebuild` finishes the local rebuild
- THEN the command exits non-zero
- AND it names the output mismatch
- AND it does not write publishable witness sidecars
