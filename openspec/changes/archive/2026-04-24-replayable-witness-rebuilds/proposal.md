# Replayable witness rebuilds

## Why

crunch now has the publisher half of witnessed self-hosting:

- `./scripts/prove-self-hosting.sh` can produce a full stage0 -> stage1 ->
  stage2 proof bundle,
- `crunch release create` and `crunch release verify` package and validate that
  bundle as release evidence,
- `crunch release attest`, `crunch attest policy-init`, `crunch release
  witness-export`, `crunch attest witness-create`, and `crunch attest
  witness-import` cover the attestation and handoff surfaces.

What is still missing is the witness-side rebuild rail. The second environment
still has to improvise its own rebuild recipe, scratch layout, and evidence
capture before it can call `crunch attest witness-create`. That gap is now the
highest-leverage blocker between today's bounded witnessed workflow and the
README roadmap item to add independent rebuild evidence for release outputs.

This change does not claim a full-source bootstrap root or globally
reproducible releases. It only turns the witness-side rebuild into one
checked-in, replayable workflow.

## What Changes

- **Add a request-driven witness rebuild command.** Introduce `crunch release
  witness-rebuild <request-dir>` to validate an exported witness request,
  replay the recorded rebuild workflow, and emit witness sidecars plus rebuild
  audit artifacts.
- **Define a deterministic witness rebuild plan core.** Parse the request
  layout, release attestation, and release-evidence manifest into one bounded,
  fail-closed plan instead of letting the witness improvise file paths and
  output matching.
- **Add a checked-in witness-side helper.** Provide
  `./scripts/rebuild-witness-request.sh` to perform host preflight and wrap the
  new CLI so the witness environment can run the rebuild with the same kind of
  ergonomics the repo already gives the publisher proof path.
- **Document and test the full publisher -> witness-runner -> publisher rail.**
  Keep the trust claim bounded to external witness agreement under configured
  policy, not full-source bootstrap or global reproducibility.

## Non-Goals

- Replacing the musl.cc-derived reduced seed provider.
- Removing the remaining stage0 host-tool prerequisites from the proof path.
- Introducing hosted witness discovery, transparency logs, or remote policy
  distribution.
- Claiming that one matching witness proves full-source bootstrap or globally
  reproducible release artifacts.

## Capabilities

### New Capabilities
- `witness-request-rebuild-runner`: rebuild one exported witness request into
  witness sidecars and rebuild evidence through a checked-in command path.
- `witness-rebuild-audit-evidence`: persist enough rebuild metadata to explain
  how the witness-side outputs were produced before they are imported.

### Modified Capabilities
- `witnessed-self-hosting-operator-rail`: the second environment no longer
  hand-assembles its rebuild recipe; it follows one checked-in replay path.

## Impact

- **Files**: `src/main.rs`, `src/release_cmd.rs`, witness/release helper
  modules, `scripts/`, `tests/release_cli.rs`, `README.md`, and
  `docs/operator-workflows.md`.
- **APIs**: new `crunch release witness-rebuild` CLI surface and the checked-in
  witness-side wrapper script `./scripts/rebuild-witness-request.sh`.
- **Dependencies**: no new network protocol or trust service dependency.
- **Testing**: request-validation negatives, rebuild-output mismatch coverage,
  and one end-to-end publisher -> witness -> publisher workflow test.

## Constraints

- The rebuild runner MUST derive its plan from exported request artifacts,
  release evidence, and release-attestation workflow identity instead of an ad
  hoc operator recipe.
- The workflow MUST fail before rebuild when the request schema, layout, or
  recorded workflow identity is unsupported.
- The workflow MUST keep verified-self-hosting claims narrower than
  full-source bootstrap or global reproducibility.
- The first implementation MUST include both positive and negative test
  coverage.

## Traceability

| Proposal slice | Delta spec |
|---|---|
| Checked-in witness rebuild runner for exported request directories | `specs/release-verification-tech/spec.md` |
| Updated witnessed-self-hosting workflow and docs | `specs/release-evidence/spec.md` |

## How to validate

1. `openspec validate replayable-witness-rebuilds` succeeds.
2. `openspec_gate stage=proposal change=replayable-witness-rebuilds` passes.
3. `cargo test -p crunch --test release_cli witness_rebuild_ -- --nocapture`
   proves request validation, unsupported-workflow rejection, and rebuild-path
   negatives fail closed while the happy path succeeds.
4. `cargo test -p crunch --test release_cli witnessed_self_hosting_rebuild_ --
   --nocapture` proves the publisher -> witness-runner -> publisher rail ends
   with `technical_class=external-witness-match`, `policy_status=satisfied`,
   and `final_class=quorum-satisfied`.
5. `openspec_gate stage=design change=replayable-witness-rebuilds` and
   `openspec_gate stage=tasks change=replayable-witness-rebuilds` pass once the
   design and task packet are complete.
