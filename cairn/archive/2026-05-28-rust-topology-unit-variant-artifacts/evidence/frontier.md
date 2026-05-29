# Frontier evidence: unit variant artifact identity

## Question

What is the next native Rust topology frontier after preserving transitive search paths?

## Inspected evidence

Source: `target/mantle-self-rust-plan-probe-transitive-search-post-commit/blocker-summary.txt` from implementation commit `28f26636a437031b4e3e345ab397ee9a72e2f556`.

```text
topology_execution_status=blocked
executions=611
blocker={"class":"rustc-failed","message":"error E0463: can't find crate for `crunch_store`\n --> ./crates/crunch-build/src/ca_mapping.rs:5:9\n...\nerror: found crates (`snix_castore` and `snix_castore`) with colliding StableCrateId values\n --> ./crates/crunch-build/src/fod.rs:5:5\n..."}
```

Additional receipt inspection:

```text
crunch-build units: 30 succeeded, 6 failed
crunch-store lib units: 15 and 39 both succeeded
snix-castore lib units: 20 and 512 both succeeded
snix-castore custom-build units: 519 and 613 both succeeded
```

## Decision

Create a Cairn change for unit-variant-aware artifact binding. Package ID alone is too coarse for direct `--extern` binding and too broad for rustc search-path scope once duplicate same-package variants exist.

## Owner

Current agent / Mantle maintainer.

## Next action

Implement the change with focused same-package variant tests before rerunning the clean self-probe.
