## Implementation / evidence

- [x] [serial] I1 Export a public-only witness request for the current provider-bound release evidence bundle and record request path plus BLAKE3 digest. r[release_witness.external_handoff]
  Evidence: `evidence/external-witness-blocker.md` records the original blocked request; `evidence/external-witness-success-2026-07-02.md` records the replacement request digest `1f5f4f527bcd0499b98682f8f62189a7c341477f58d946aab865876d4a30eeb5` after the source-policy fix.
- [x] [serial] I2 Send the request to an independently operated witness and record the witness identity, returned sidecar paths, and operator-independence basis. r[release_witness.external_handoff]
  Evidence: `evidence/external-witness-success-2026-07-02.md` records Aspen host `aspen1`, identity `aspen1-external-witness`, signer `crunch-aspen1-1`, returned sidecars, and unshare/FUSE rebuild status `exit_status=0`.
- [x] [serial] I3 Import the returned witness sidecars and record the final `mantle attest release-verify` status, counted witness class, and bounded non-claims. r[release_witness.external_handoff]
  Evidence: `evidence/external-witness-success-2026-07-02.md` records successful import and final `independent_agreement_class: independent-rebuild-agreement` with one policy-counted Aspen witness.
- [x] [serial] I4 Update release notes or operator docs so provider-bound local quorum and external independent witness agreement use distinct wording. r[release_witness.external_handoff]
  Evidence: `docs/release-notes/provider-bound-release-evidence-2026-07-02-source-policy-fixed.md` records the bounded claim and distinguishes the superseded blocked 2026-06-28 request from the fresh external witness agreement.

## Verification

- [x] [serial] V1 Positive: verify that the imported external witness is signature-valid, policy-counted, release-digest-matched, and rebuilt-digest-matched. r[release_witness.external_handoff]
  Evidence: `evidence/external-witness-success-2026-07-02.md` records `signature_valid: true`, `digest_match: true`, `policy_counted: true`, `classification_reason: counted`.
- [x] [serial] V2 Negative: verify at least one fail-closed path such as missing signature, wrong release digest, conflicting duplicate identity, unknown key, or policy-insufficient witness set. r[release_witness.external_handoff]
  Evidence: `evidence/external-witness-success-2026-07-02.md` records missing-signature import failure with `exit_status=3`.
- [x] [serial] V3 Run `mantle attest release-verify` with the final trusted key set, `git diff --check`, and Cairn validation/gates; record evidence before archive. r[release_witness.external_handoff]
  Evidence: `evidence/external-witness-success-2026-07-02.md` records final release verification, `git diff --check`, `cairn validate`, and proposal/design/tasks gate PASS results.
