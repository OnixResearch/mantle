## Implementation

- [x] [serial] I1 Add pure redacted status/report models for worker snapshots, queue phases, route decisions, upload summaries, transfer summaries, trust basis, logs, and non-claims. r[remote_builds.operator_observability]
- [x] [serial] I2 Extend `mantle remote status` human/JSON output with bounded worker, ticket, queue, active job, recent failure, capability, and log-cursor fields. r[remote_builds.operator_observability]
- [x] [serial] I3 Extend build human/JSON reports with route, rejected reasons, endpoint, upload classes/bytes, delta/full transfer, fallback reason, signer/trust basis, attestation paths, and non-claims. r[remote_builds.operator_observability]
- [x] [serial] I4 Enforce redaction and size limits for logs, argv/path lists, untrusted diagnostics, raw env values, private key paths, uploaded content, and bearer tickets. r[remote_builds.operator_observability]

## Verification

- [x] [serial] V1 Positive: status and build JSON reports include route, queue, upload, transfer, trust, and attestation summaries with stable field names. r[remote_builds.operator_observability]
- [x] [serial] V2 Negative: bearer tickets, private key paths, raw env values, uploaded content, oversized logs, and untrusted log control bytes are omitted or bounded. r[remote_builds.operator_observability]
- [x] [serial] V3 Run focused status/report schema tests plus Cairn validate and proposal/design/tasks gates for this change. r[remote_builds.operator_observability]
