## Context

The repo already supports the technical pieces of decentralized release
verification:

- `crunch release create` and `crunch release verify` package and validate full
  self-hosting proof bundles,
- `crunch release attest` creates a signed release attestation in a verification
  directory, and
- `crunch attest witness-create` plus `crunch attest release-verify` already
  model matching witnesses, policy sufficiency, and final classes.

The operational gap is policy authoring and workflow cohesion. The verifier
expects `policy.json` and optional `revocations.json`, but the checked-in CLI
has no command to create those files. Tests synthesize policy JSON directly and
operator docs describe verification at a higher level, which means the first
real witnessed-self-hosting workflow is still partly out-of-band.

This change keeps the technical and social planes separate. It does not invent
new attestation formats or stronger trust claims. It only turns the existing
first-phase model into a checked-in workflow an operator can actually run.

## Goals / Non-Goals

**Goals**
- Add a CLI way to scaffold verifier-local policy material.
- Keep the policy scaffolding deterministic and directly compatible with the
  existing verifier.
- Document one narrow witnessed self-hosting workflow from verified release
  bundle to witness verification result.
- Cover both happy-path and failure-path behavior with targeted CLI tests.

**Non-Goals**
- Changing release- or witness-attestation digest formats.
- Adding hosted policy distribution, witness discovery, or transparency logs.
- Proving that one machine-local integration test equals real-world social
  independence.
- Reducing the stage0 bootstrap root in this change.

## Decisions

### 1. Put policy scaffolding under `crunch attest`

**Choice:** add `crunch attest policy-init <verification-dir>` instead of a
`crunch release` subcommand.

**Rationale:** the command operates on the same verification directory surface
as `witness-create`, `release-show`, `witness-show`, and `release-verify`.
Keeping it under `attest` makes the lifecycle coherent: `release attest`
creates the release attestation, and the `attest` subcommands manage the rest of
that verification directory.

**Alternative:** add `crunch release policy-init`.

**Why not:** policy material is verifier-local social configuration, not a
property of the original release bundle. Grouping it under `release` would blur
that boundary.

### 2. Start with named profiles, not raw JSON-only flags

**Choice:** the first CLI slice will support named profiles:
- `self-proof-only`
- `single-witness`

and allow explicit signer/identity inputs for each profile.

**Rationale:** named profiles keep the first operator workflow small and reduce
JSON footguns. They also match the current repo goal: operationalize one narrow
witnessed-self-hosting path before generalizing every possible policy shape.

**Alternative:** require operators to provide every field as separate CLI flags
or keep policy creation as manual JSON editing.

**Why not:** that keeps the main adoption blocker in place. The repo already has
working policy structs; the missing piece is a safe first-class authoring path.

### 3. Refuse to clobber policy files by default

**Choice:** `policy-init` will fail if `policy.json` or `revocations.json`
already exists unless the operator passes an explicit overwrite flag.

**Rationale:** verifier-local policy files are safety-critical social inputs.
Silent overwrite would make it too easy to replace an existing policy or clear a
revocation file accidentally.

**Alternative:** always rewrite the files.

**Why not:** the convenient path is too risky for a command that changes trust
policy.

### 4. Keep revocations file explicit and deterministic

**Choice:** the scaffolding command always writes an explicit empty
`revocations.json` for the documented profiles.

**Rationale:** the verifier already treats revocations as an optional separate
artifact, but the operator workflow is simpler when the directory starts in one
fully-initialized state. An explicit empty file is easier to inspect, copy, and
later edit than an omitted file whose absence might mean either "not yet set up"
or "intentionally empty".

### 5. Validate the workflow end-to-end with CLI integration tests

**Choice:** add targeted integration coverage in `tests/release_cli.rs` for:
- positive `policy-init` profile generation,
- negative no-clobber behavior, and
- an end-to-end witnessed-self-hosting path that uses `release attest`,
  `attest policy-init`, `attest witness-create`, and `attest release-verify`.

**Rationale:** the repo already has release CLI fixtures, so this is the
cheapest honest place to prove the operator flow works without inventing a new
harness.

**Alternative:** rely on unit tests or docs alone.

**Why not:** this milestone is about operationalizing a workflow. The CLI path
itself needs direct coverage.

## CLI shape

Planned shell surface:

```text
crunch attest policy-init <verification-dir> \
  --profile <self-proof-only|single-witness> \
  --trusted-release-signer <name>... \
  [--trusted-witness-identity <identity>...] \
  [--force]
```

Initial profile semantics:

- `self-proof-only`
  - `min_matching_witnesses = 0`
  - `independence_field = "witness_identity"`
  - empty `trusted_witness_signers`
- `single-witness`
  - `min_matching_witnesses = 1`
  - `independence_field = "witness_identity"`
  - one or more `trusted_witness_signers`

Both profiles require at least one trusted release signer name.

## Data flow

```text
verified release-evidence bundle
  -> `crunch release attest`
  -> verification dir with release-attestation.json + .sig
  -> `crunch attest policy-init`
  -> verification dir gains policy.json + revocations.json
  -> `crunch attest witness-create`
  -> verification dir gains witnesses/<identity>.json + .sig
  -> `crunch attest release-verify`
  -> technical class + policy status + final class
```

## Verification strategy

- `policy-init` positive tests for `self-proof-only` and `single-witness`
  output.
- `policy-init` negative test for existing-file no-clobber behavior.
- end-to-end workflow test that proves a matching witness raises the technical
  class to `external-witness-match` and, with the scaffolded single-witness
  policy, the final class to `quorum-satisfied`.
- docs validation that the witnessed-self-hosting workflow keeps claims bounded
  to external witness agreement under configured policy rather than full-source
  bootstrap.

## Risks / Trade-offs

**Profile set may feel too narrow.**
That is acceptable for the first slice. The goal is one good default workflow,
not every policy permutation.

**A local CLI test cannot prove real-world independence.**
Correct. The test only proves the technical and policy machinery. Docs must keep
that distinction explicit.

**Policy authoring under `attest` adds one more subcommand family.**
That is still simpler than asking operators to edit JSON by hand or discover the
schema from tests.
