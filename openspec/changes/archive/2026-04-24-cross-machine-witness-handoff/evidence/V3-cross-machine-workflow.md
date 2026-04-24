# V3 cross-machine workflow

## Command

`cargo test -p crunch --test release_cli cross_machine_witness_handoff_ -- --nocapture`

## Output

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
     Running tests/release_cli.rs (/tmp/crunch-release-cli-new/debug/deps/release_cli-5f49c1ce9d3c266b)

running 1 test
test cross_machine_witness_handoff_reports_quorum_satisfied ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 39 filtered out; finished in 0.08s
```

## Command

`openspec validate cross-machine-witness-handoff`

## Output

```text
Change 'cross-machine-witness-handoff' is valid
```

## Command

`openspec_gate stage=design change=cross-machine-witness-handoff`

## Output

```text
VERDICT: PASS
```

## Command

`openspec_gate stage=tasks change=cross-machine-witness-handoff`

## Output

```text
VERDICT: PASS
```
