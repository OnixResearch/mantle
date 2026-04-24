# Evidence: V1 witness rebuild CLI

Task-ID: V1
Covers: release.verification.tech.witness.rebuild.cli
Status: passed

## Commands

```bash
bash -n scripts/rebuild-witness-request.sh
cargo test -p crunch --test release_cli witness_rebuild_ -- --nocapture
```

## Results

- `bash -n scripts/rebuild-witness-request.sh` exited successfully.
- `cargo test -p crunch --test release_cli witness_rebuild_ -- --nocapture`
  ran 6 witness-rebuild-focused tests and passed:

```text
running 6 tests
test witness_rebuild_cli_rejects_unsupported_workflow_before_running_driver ... ok
test witness_rebuild_cli_rejects_published_output_name_mismatch_before_running_driver ... ok
test witness_rebuild_cli_check_is_preflight_only ... ok
test witness_rebuild_cli_happy_path_writes_sidecars_and_audit ... ok
test witness_rebuild_cli_rejects_rebuilt_output_digest_mismatch ... ok
test witness_rebuild_helper_check_is_preflight_only ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 41 filtered out; finished in 0.22s
```

## Coverage notes

This command proves:
- happy-path `crunch release witness-rebuild` success,
- helper `--check` preflight-only behavior,
- unsupported workflow rejection before driver launch,
- published-output-name mismatch rejection before driver launch, and
- rebuilt-output digest mismatch rejection with failed audit metadata.
