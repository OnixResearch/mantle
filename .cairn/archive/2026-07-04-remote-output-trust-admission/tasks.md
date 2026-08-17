## Implementation

- [x] [serial] I1 Implement a pure remote-output admission model for requested identity, store prefix, PathInfo signatures, object refs, artifact attestations, key-material trust, revocation, and policy hash. r[remote_builds.output_trust_admission_pipeline]
- [x] [serial] I2 Use admission preflight in remote route planning and client dispatch when builder output keys or attestation authorities are known. r[remote_builds.output_trust_admission_pipeline]
- [x] [serial] I3 Use final admission before persisting or exporting any remote output, including stdio, SSH-stdio, coordinator, and future P2P paths. r[remote_builds.output_trust_admission_pipeline]
- [x] [serial] I4 Render trust-basis reports with key-material digests and redacted secret-bearing fields. r[remote_builds.output_trust_admission_pipeline]

## Verification

- [x] [serial] V1 Positive: trusted builder key plus matching PathInfo, object refs, store prefix, and attestation imports a remote output. r[remote_builds.output_trust_admission_pipeline]
- [x] [serial] V2 Negative: resource ticket without output trust, unknown key, same-name-different-key, revoked/expired key, wrong store prefix, stale output identity, missing attestation, and tampered object are rejected. r[remote_builds.output_trust_admission_pipeline]
- [x] [serial] V3 Run focused trust-admission/import tests plus Cairn validate and proposal/design/tasks gates for this change. r[remote_builds.output_trust_admission_pipeline]
