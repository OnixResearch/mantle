# V1 witness export

## Command

`cargo test -p crunch --test release_cli witness_export_ -- --nocapture`

## Output

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.23s
     Running tests/release_cli.rs (/tmp/crunch-release-cli-new/debug/deps/release_cli-5f49c1ce9d3c266b)

running 2 tests
test witness_export_rejects_release_id_mismatch ... ok
test witness_export_writes_request_directory ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 38 filtered out; finished in 0.03s

```
