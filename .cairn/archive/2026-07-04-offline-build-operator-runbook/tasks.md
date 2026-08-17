## Implementation

- [x] [serial] I1 Add an offline build runbook to README/operator docs with exact commands for source-bundle export, import with pinning, verify/preflight, `--offline-source-preflight`, `--no-substitute`, build/run, and evidence inspection. r[operator_diagnostics.offline_build_runbook]
- [x] [serial] I2 Update root command references and CLI help coverage so `mantle source bundle` and `--offline-source-preflight` are discoverable from normal operator docs. r[operator_diagnostics.offline_build_runbook]
- [x] [serial] I3 Add bounded remediation hints for missing, stale, unpinned, unsupported, untrusted, network-required source state and malformed offline Cargo evidence while preserving parseable JSON output. r[operator_diagnostics.offline_build_runbook]
- [x] [serial] I4 Document current vs future behavior for source-bundle preflight and source-bundle route execution so the runbook does not claim unavailable execution semantics. r[operator_diagnostics.offline_build_runbook]
- [x] [serial] I5 Add docs/tests that reject missing offline commands, stale examples, and overclaiming language around source readiness, offline Cargo evidence, route plans, and build success. r[operator_diagnostics.offline_build_runbook]

## Verification

- [x] [serial] V1 Positive: docs/runbook tests find the source-bundle export/import/preflight/build/evidence command sequence and required evidence field names. r[operator_diagnostics.offline_build_runbook]
- [x] [serial] V2 Positive: source-preflight and offline Cargo diagnostics include stable next-action hints in human output and documented fields in JSON output. r[operator_diagnostics.offline_build_runbook]
- [x] [serial] V3 Negative: docs containing source-bundle readiness equals build success, offline Cargo equals Cargo-free execution, route eligibility equals output trust, or source import equals reproducibility are rejected by claim-boundary tests. r[operator_diagnostics.offline_build_runbook]
- [x] [serial] V4 Negative: JSON diagnostics remain parseable and do not include raw environment values, bearer tokens, private key paths, or unbounded source lists. r[operator_diagnostics.offline_build_runbook]
- [x] [serial] V5 Run focused docs/operator diagnostics tests plus `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .`, proposal gate, design gate, and tasks gate for this change. r[operator_diagnostics.offline_build_runbook]
