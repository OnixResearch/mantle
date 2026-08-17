# Current blocker

From clean package metadata env probe `target/mantle-self-rust-plan-probe-after-0efb3458-clean/blocker-summary.txt`:

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
thread 'main' (969811) panicked at /home/brittonr/git/mantle/vendor-deps/aws-lc-sys/builder/main.rs:160:39:
missing env var "CARGO_CFG_TARGET_ARCH"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Baseline focused tests before this change:

```text
running 70 tests
...
test result: ok. 70 passed; 0 failed; 0 ignored; 0 measured; 427 filtered out; finished in 0.03s
```
