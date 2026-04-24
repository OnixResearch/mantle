# V2 witness import

## Command

`cargo test -p crunch --test release_cli witness_import_ -- --nocapture`

## Output

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running tests/release_cli.rs (/tmp/crunch-release-cli-new/debug/deps/release_cli-5f49c1ce9d3c266b)

running 4 tests
test witness_import_rejects_wrong_release_attestation_digest ... ok
test witness_import_rejects_missing_signature_sidecar ... ok
test witness_import_accepts_directory_source_and_skips_exact_duplicates ... ok
test witness_import_rejects_conflicting_duplicate_identity ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 36 filtered out; finished in 0.03s

```
