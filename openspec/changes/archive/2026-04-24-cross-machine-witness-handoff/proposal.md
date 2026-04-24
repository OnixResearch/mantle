# cross-machine-witness-handoff

## Why

The repo now has a complete local witnessed-self-hosting operator rail:
`release verify`, `release attest`, `attest key-show`, `attest policy-init`,
`attest witness-create`, and `attest release-verify` all work together.

What it still lacks is a safe, repeatable handoff between the publisher and an
actual second environment. Today a real witness run still depends on ad hoc
copying of the release bundle, the release-attestation seed, and the returned
`witnesses/` sidecars. That is the highest-leverage remaining gap between
"machine-local witnessed workflow" and "publishable external witness result".

## What Changes

- **Add a publisher-side witness request export.** Introduce `crunch release
  witness-export` to copy one verified release-evidence bundle plus the signed
  `release-attestation.json` seed into a portable witness-request directory.
- **Add a publisher-side witness import.** Introduce `crunch attest
  witness-import` to validate and copy returned witness-attestation sidecars
  into the publisher's verification directory without hand-managed file moves.
- **Document the cross-machine workflow.** Show one bounded publisher ->
  witness -> publisher command chain that uses the exported request directory,
  keeps key ownership local, and preserves the existing narrow trust claims.
- **Add positive and negative integration coverage.** Prove export layout,
  import validation, and an end-to-end publisher/witness exchange path.

## Non-Goals

- Running an actual remote transport protocol or witness-discovery service.
- Changing release-attestation or witness-attestation canonical formats.
- Auto-copying private signing keys into the witness-request bundle.
- Claiming that a portable witness handoff alone proves full-source bootstrap
  or globally reproducible releases.

## Capabilities

### New Capabilities
- `release.evidence.workflow.witnessed.request-export`: export a verified
  release bundle plus release-attestation seed into a portable witness-request
  directory for a second environment.
- `release.verification.tech.witness.import.cli`: import returned
  witness-attestation sidecars into a publisher verification directory with
  release-digest and duplicate-safety checks.
- `release.evidence.workflow.witnessed.crossmachine.docs`: document the
  publisher -> witness -> publisher handoff as the checked-in external-witness
  workflow.

## Impact

- **Files**: `src/main.rs`, `src/release_cmd.rs`, `src/attest_cmd.rs`,
  release-evidence / release-attestation helpers, `tests/release_cli.rs`, and
  release/operator docs.
- **APIs**: new `crunch release witness-export` and `crunch attest
  witness-import` CLI surfaces.
- **Dependencies**: no new external dependencies.
- **Testing**: targeted release CLI integration coverage for export/import and
  one end-to-end cross-machine handoff simulation.

## Constraints

- The exported witness-request directory MUST contain only public verification
  material: verified release-evidence contents, the signed release attestation,
  and request metadata. It MUST NOT copy signing keys.
- Witness import MUST fail closed on missing signature sidecars, release-digest
  mismatches, or conflicting existing witness identities.
- The documented workflow MUST keep "verified self hosting" claims bounded to
  external witness agreement under configured policy.

## Traceability

| Proposal slice | Delta spec |
|---|---|
| Portable witness-request export | `specs/release-evidence/spec.md` |
| Safe witness import CLI | `specs/release-verification-tech/spec.md` |

## How to validate

1. `openspec validate cross-machine-witness-handoff` succeeds.
2. `openspec_gate stage=proposal change=cross-machine-witness-handoff` passes.
3. `cargo test -p crunch --test release_cli witness_export_ -- --nocapture`
   proves the portable request layout and negative export checks.
4. `cargo test -p crunch --test release_cli witness_import_ -- --nocapture`
   proves witness import accepts matching sidecars and rejects mismatches,
   missing signatures, and conflicting duplicates.
5. `cargo test -p crunch --test release_cli cross_machine_witness_handoff_ --
   --nocapture` proves a publisher export, a witness-side attestation created
   from the exported seed, an import back into the publisher verification dir,
   and a final `attest release-verify` result of
   `technical_class=external-witness-match`, `policy_status=satisfied`, and
   `final_class=quorum-satisfied`.
6. `openspec_gate stage=design change=cross-machine-witness-handoff` and
   `openspec_gate stage=tasks change=cross-machine-witness-handoff` pass once
   implementation and evidence are in place.
