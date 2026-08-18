## Implementation

- [x] [serial] I1 Define the witness expansion report schema and pure classifier for requested, returned, skipped, failed, and policy-counted witnesses. r[verification_evidence.external_witness_expansion]
  - Evidence: release independent-agreement reports classify counted/skipped/failed witnesses with signer, digest, source-mode, and policy fields; see `evidence/validation.md`.
- [x] [serial] I2 Extend release/global evidence preparation to preserve witness identity, signer key, independence domain, host class, source mode, digest match, and policy decision. r[verification_evidence.external_witness_expansion]
  - Evidence: per-witness release verification output and release-derived global evidence now carry those fields; see `evidence/validation.md`.
- [x] [serial] I3 Add operator-facing summaries for quorum profiles and witness coverage without promoting unverified witnesses. r[verification_evidence.external_witness_expansion]
  - Evidence: release verification JSON names the independence field, required witness count, counted/skipped/failed counts, and classified witnesses; see `evidence/validation.md`.

## Verification

- [x] [serial] V1 Positive: import multiple valid external witnesses from independent domains and prove the configured quorum is satisfied. r[verification_evidence.external_witness_expansion]
  - Evidence: `cargo test -p mantle --test release_cli attest_release_verify -- --nocapture` includes `attest_release_verify_reports_two_independent_witness_agreement` passing.
- [x] [serial] V2 Negative: include same-domain, unknown-key, bad-signature, stale-request, and wrong-release-digest witnesses and prove they are skipped or failed without inflating quorum. r[verification_evidence.external_witness_expansion]
  - Evidence: focused release verification tests cover same-host-class/same-domain, unknown-key, invalid-signature, revoked, malformed, and wrong-release-digest classifications without quorum inflation; focused witness-import tests reject wrong-release/stale-request sidecars before import.
- [x] [serial] V3 Run focused release verification tests, witness report tests, docs checks, and Cairn validate/gates for this change. r[verification_evidence.external_witness_expansion]
  - Evidence: focused cargo/rustfmt checks are recorded in `evidence/validation.md`; Cairn gates and validation are run after task completion.
