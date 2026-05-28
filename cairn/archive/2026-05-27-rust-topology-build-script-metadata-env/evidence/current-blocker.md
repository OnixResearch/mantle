# Current blocker

From final proc-macro topology probe `target/mantle-self-rust-plan-probe-after-573bc89b-clean/blocker-summary.txt`:

```text
topology_execution=blocked
topology_unit_executions=15
metadata_runs=6

async-stream-impl unit execution:
registry+https://github.com/rust-lang/crates.io-index#async-stream-impl@0.3.6 target=proc-macro status=success reason=

blocker classes:
      1 build-script-run-failed

topology blocker:
- build-script-run-failed:
thread 'main' (732424) panicked at /home/brittonr/git/mantle/vendor-deps/aws-lc-rs/build.rs:110:5:
missing DEP_AWS_LC_ include
```

`vendor-deps/aws-lc-rs/build.rs` searches `DEP_AWS_LC_..._INCLUDE` and re-exports selected metadata. `vendor-deps/aws-lc-sys/Cargo.toml` declares `links = "aws_lc_0_39_1"` and its build script emits metadata such as `cargo:include=...`.
