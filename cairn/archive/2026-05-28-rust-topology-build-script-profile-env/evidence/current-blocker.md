# Current blocker

From clean target cfg env probe `target/mantle-self-rust-plan-probe-after-c9bf3938-clean/blocker-summary.txt`:

```text
topology_execution=blocked
topology_unit_executions=30
metadata_runs=12

aws-lc related unit executions:
registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1 target=build-script-build kind=custom-build status=success

blocker classes:
      1 build-script-run-failed

topology blocker:
- build-script-run-failed:

error occurred in cc-rs: environment variable OPT_LEVEL not defined
```

Baseline focused tests before this change:

```text
running 72 tests
...
test result: ok. 72 passed; 0 failed; 0 ignored; 0 measured; 427 filtered out; finished in 0.03s
```
