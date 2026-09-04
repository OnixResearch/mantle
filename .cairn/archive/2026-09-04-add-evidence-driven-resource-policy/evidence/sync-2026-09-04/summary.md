# Resource-policy accepted-spec sync

Date: 2026-09-04

Cairn dry-run and execute sync passed. The mutation added exactly these seven requirements to `.cairn/specs/remote-builds/spec.md`:

- `remote_builds.resource_observations`
- `remote_builds.replayable_resource_selection`
- `remote_builds.positive_oom_retry`
- `remote_builds.usage_reservation_and_reconciliation`
- `remote_builds.authorized_result_sharing`
- `remote_builds.resource_benchmark_evidence`
- `remote_builds.resource_policy_rollout`

The accepted spec changed from BLAKE3 `5080bcf858b796403fb2c680d07021fb83d12353c78bad5830089078211e699f` to `16bef677017870273ff7a5034a6b830d5897a408af40572681bc89e163dfd2dd`.

Post-sync Cairn validation and the tasks gate passed. Tracey reported `157/157` referenced under profile `mantle-default`.
