## Implementation / evidence

- [x] [serial] I1 Export a public-only witness request for the current provider-bound release evidence bundle and record request path plus BLAKE3 digest. r[release_witness.external_handoff]
- [ ] [serial] I2 Send the request to an independently operated witness and record the witness identity, returned sidecar paths, and operator-independence basis. r[release_witness.external_handoff]
- [ ] [serial] I3 Import the returned witness sidecars and record the final `mantle attest release-verify` status, counted witness class, and bounded non-claims. r[release_witness.external_handoff]
- [ ] [serial] I4 Update release notes or operator docs so provider-bound local quorum and external independent witness agreement use distinct wording. r[release_witness.external_handoff]

## Verification

- [ ] [serial] V1 Positive: verify that the imported external witness is signature-valid, policy-counted, release-digest-matched, and rebuilt-digest-matched. r[release_witness.external_handoff]
- [ ] [serial] V2 Negative: verify at least one fail-closed path such as missing signature, wrong release digest, conflicting duplicate identity, unknown key, or policy-insufficient witness set. r[release_witness.external_handoff]
- [ ] [serial] V3 Run `mantle attest release-verify` with the final trusted key set, `git diff --check`, and Cairn validation/gates; record evidence before archive. r[release_witness.external_handoff]
