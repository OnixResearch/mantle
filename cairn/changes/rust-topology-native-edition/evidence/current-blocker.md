# Current blocker evidence

Source: `target/mantle-self-rust-plan-probe-after-b3ed3a8e-clean/blocker-summary.txt` from pueue task `14` of the previous change.

```text
probe: target/mantle-self-rust-plan-probe-after-b3ed3a8e-clean/receipt.json
head: b3ed3a8e415581d41b2f4896da7f2d51ed13001a
git_status_short_bytes=0

probe_status=0
topology_execution=blocked
topology_unit_executions=2
metadata_runs=1

blocker classes:
      2 rustc-failed

topology blocker:
- rustc-failed: error: let chains are only allowed in Rust 2024 or later
  --> ./vendor/nix-compat-derive/src/de.rs:39:8

error: let chains are only allowed in Rust 2024 or later
  --> ./vendor/nix-compat-derive/src/ser.rs:38:8
```

Decision: begin a bounded native edition propagation change rather than patching vendored source syntax. The manifest already declares `edition = "2024"`; Mantle's native rustc args are wrong.
