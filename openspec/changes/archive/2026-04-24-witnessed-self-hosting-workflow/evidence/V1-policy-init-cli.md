# V1 Policy-init CLI evidence

## Commands

```bash
cargo test -p crunch --bin crunch build_policy_file_contents_ -- --nocapture
cargo test -p crunch --test release_cli attest_policy_init_ -- --nocapture
```

## Results

- `cargo test -p crunch --bin crunch build_policy_file_contents_ -- --nocapture`
  - `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 172 filtered out; finished in 0.01s`
  - Covered pure profile generation for:
    - `build_policy_file_contents_self_proof_only_drops_witnesses`
    - `build_policy_file_contents_self_proof_only_rejects_identity`
    - `build_policy_file_contents_single_witness_requires_identity`
- `cargo test -p crunch --test release_cli attest_policy_init_ -- --nocapture`
  - `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 26 filtered out; finished in 0.04s`
  - Covered CLI behavior for:
    - `attest_policy_init_self_proof_only_writes_policy_files`
    - `attest_policy_init_single_witness_writes_policy_files`
    - `attest_policy_init_rejects_existing_policy_without_force`

## Summary

Both supported policy profiles now scaffold verifier-local `policy.json` /
`revocations.json` successfully, and the no-clobber negative path fails with
`refusing to overwrite existing ... without --force`.
