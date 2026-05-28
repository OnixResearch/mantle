# Current blocker evidence

Source: pueue task `29`, `target/mantle-self-rust-plan-probe-after-50bae139-clean/blocker-summary.txt`.

```text
probe: target/mantle-self-rust-plan-probe-after-50bae139-clean/receipt.json
head: 50bae139d502f085e3402bcfbf1a7fa46f6d2452
git_status_short_bytes=0

probe_status=0
native_package_target_ready=true
native_host_unit_graph_ready=true
topology_execution=blocked
topology_unit_executions=1
metadata_runs=0

first nix-compat-derive editions:
643:path+file:///home/brittonr/git/mantle/vendor/nix-compat-derive#0.1.0:nix-compat-derive:proc-macro:build edition=2024

blocker classes:
      1 build-script-run-failed

topology blocker:
- build-script-run-failed: Environment variable $RUSTC is not set during execution of build script
```

Decision: begin a bounded build-script execution environment change. Later archive/AGENTS commits after `50bae139` do not alter `src/rust_plan.rs`, so this remains the code frontier.
