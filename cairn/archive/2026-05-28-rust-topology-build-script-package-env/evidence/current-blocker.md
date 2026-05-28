# Current blocker

From clean linked metadata repair probe `target/mantle-self-rust-plan-probe-after-267c2259-clean/blocker-summary.txt`:

```text
topology_execution=blocked
topology_unit_executions=24
metadata_runs=7

aws-lc related unit executions:
registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1 target=build-script-build kind=custom-build status=failed

blocker classes:
      1 rustc-failed

topology blocker:
- rustc-failed: error: environment variable `CARGO_PKG_VERSION` not defined at compile time
   --> /home/brittonr/git/mantle/vendor-deps/aws-lc-sys/builder/main.rs:311:23
    |
311 | const VERSION: &str = env!("CARGO_PKG_VERSION");
    |                       ^^^^^^^^^^^^^^^^^^^^^^^^^
```
