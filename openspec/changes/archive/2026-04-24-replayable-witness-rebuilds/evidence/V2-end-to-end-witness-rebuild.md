# Evidence: V2 end-to-end witness rebuild

Task-ID: V2
Covers: release.evidence.workflow.witnessed.selfhosting, release.evidence.workflow.witnessed.crossmachine.docs, release.verification.tech.witness.rebuild.cli
Status: passed

## Command

```bash
cargo test -p crunch --test release_cli witnessed_self_hosting_rebuild_ -- --nocapture
```

## Result

```text
running 1 test
test witnessed_self_hosting_rebuild_workflow_reports_quorum_satisfied ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 46 filtered out; finished in 0.19s
```

## Coverage notes

The end-to-end workflow test starts from a valid release-evidence bundle,
runs `release attest`, `attest policy-init --profile single-witness`,
`release witness-export`, `release witness-rebuild`, `attest witness-import`,
and `attest release-verify`, then asserts:

- `technical_class=external-witness-match`
- `policy_status=satisfied`
- `final_class=quorum-satisfied`
