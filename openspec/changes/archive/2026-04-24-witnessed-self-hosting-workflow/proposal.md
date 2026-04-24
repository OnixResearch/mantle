# Witnessed self-hosting workflow

## Why

crunch already has three pieces of the release-trust story:

- a full stage0 -> stage1 -> stage2 self-hosting proof,
- release-evidence bundles that package that proof with tracked-worktree source
  and released binaries, and
- release/witness attestation commands plus verifier-side policy evaluation.

What it still lacks is the operator bridge that turns those pieces into a real,
repeatable "verified self-hosting" workflow. Today a contributor can produce a
release attestation and a witness attestation, but they still have to hand-write
`policy.json` and `revocations.json`, and the checked-in docs stop short of one
narrow end-to-end workflow that promotes a successful self-proof into
`external-witness-match` / `quorum-satisfied`.

That gap is now the highest-leverage next step. It does not solve the harder
bootstrap-root problem, but it does move crunch from "I can reproduce myself"
toward "someone else can independently agree with the published result under an
explicit policy".

## What Changes

- **Add a policy-scaffolding CLI.** Introduce `crunch attest policy-init` to
  create verifier-local `policy.json` and `revocations.json` files without
  hand-authoring JSON.
- **Define first checked-in policy profiles.** Start with bounded profiles for
  `self-proof-only` and `single-witness` so operators can move from self-proof
  to one matching witness with an explicit policy instead of ad hoc files.
- **Document one witnessed self-hosting workflow.** Show the exact chain from
  `crunch release verify` to `crunch release attest`, `crunch attest
  policy-init`, `crunch attest witness-create`, and `crunch attest
  release-verify`.
- **Add end-to-end CLI coverage.** Capture both positive and negative cases for
  policy initialization and for the witnessed self-hosting verification path.

## Non-Goals

- Reducing the stage0 bootstrap root or replacing the musl.cc-derived seed in
  this change.
- Automatically proving social independence between rebuilders beyond the
  configured policy fields already supported by the verifier.
- Adding a transparency log, remote witness-discovery service, or hosted policy
  service.
- Claiming that one matching witness proves full-source bootstrap or globally
  reproducible release artifacts.

## Capabilities

### New Capabilities
- `release-verification-policy-scaffold`: create first-phase policy and
  revocation files from the CLI instead of editing JSON by hand.
- `witnessed-self-hosting-operator-rail`: run one checked-in workflow that
  promotes a verified self-hosting release from `self-proof-valid` to
  `external-witness-match` / `quorum-satisfied`.

## Impact

- **Files**: `src/main.rs`, `src/attest_cmd.rs`, release-attestation helpers,
  `tests/release_cli.rs`, and release/operator docs.
- **APIs**: new `crunch attest policy-init` CLI surface plus JSON/human output
  for initialized policy material.
- **Dependencies**: no new crypto or network dependencies.
- **Testing**: targeted CLI coverage for policy profiles, overwrite failures,
  and the witnessed self-hosting path.

## Constraints

- The workflow MUST keep release-verification claims narrower than full-source
  bootstrap or global reproducibility.
- Policy scaffolding MUST write files that `crunch attest release-verify` can
  consume directly with no manual edits for the documented profiles.
- The first change MUST include both positive and negative test coverage.
- Existing release/witness artifact digests MUST remain independent from local
  policy changes.

## Traceability

| Proposal slice | Delta spec |
|---|---|
| Witnessed self-hosting workflow from verified bundle to witness verification | `specs/release-evidence/spec.md` |
| CLI policy scaffolding for verification directories | `specs/release-verification-social/spec.md` |

## How to validate

1. `openspec validate witnessed-self-hosting-workflow` succeeds.
2. `openspec_gate stage=proposal change=witnessed-self-hosting-workflow` passes.
3. `cargo test -p crunch --test release_cli attest_policy_init_ -- --nocapture`
   proves the new policy scaffolding command handles the supported profiles and
   the no-clobber negative path.
4. `cargo test -p crunch --test release_cli witnessed_self_hosting_ -- --nocapture`
   proves the end-to-end `release attest -> policy-init -> witness-create ->
   release-verify` workflow yields the expected technical and policy classes.
5. `openspec_gate stage=design change=witnessed-self-hosting-workflow` and
   `openspec_gate stage=tasks change=witnessed-self-hosting-workflow` pass once
   the design and task packet are complete.
