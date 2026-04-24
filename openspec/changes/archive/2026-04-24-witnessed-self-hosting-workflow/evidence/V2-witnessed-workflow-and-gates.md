# V2 Witnessed workflow and gates evidence

## Commands

```bash
cargo test -p crunch --test release_cli witnessed_self_hosting_ -- --nocapture
openspec validate witnessed-self-hosting-workflow
openspec_gate stage=design change=witnessed-self-hosting-workflow
openspec_gate stage=tasks change=witnessed-self-hosting-workflow
cargo test -p crunch --test release_cli attest_ -- --nocapture
```

## Results

- `cargo test -p crunch --test release_cli witnessed_self_hosting_ -- --nocapture`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.19s`
  - Verified `witnessed_self_hosting_release_workflow_reports_quorum_satisfied`
- `openspec validate witnessed-self-hosting-workflow`
  - `Change 'witnessed-self-hosting-workflow' is valid`
- `openspec_gate stage=design change=witnessed-self-hosting-workflow`
  - `PASS`
- `openspec_gate stage=tasks change=witnessed-self-hosting-workflow`
  - `PASS`
- `cargo test -p crunch --test release_cli attest_ -- --nocapture`
  - `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.10s`
  - Re-ran the broader attestation CLI regression slice after the policy-init
    changes.

## Summary

The checked-in witnessed self-hosting workflow now runs end-to-end from
`release attest` through `policy-init`, `witness-create`, and `release-verify`,
producing `technical_class=external-witness-match`, `policy_status=satisfied`,
and `final_class=quorum-satisfied`. The change remains OpenSpec-valid and both
design/tasks gates pass.
