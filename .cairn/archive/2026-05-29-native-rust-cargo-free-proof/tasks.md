# Tasks

## Spec

- [x] [serial] Add bounded Cargo-free topology proof requirement and design. r[rust_package_planning.cargo_free_topology_proof]

## Implementation

- [x] [serial] Add proof runner with fast preflight and full proof modes. r[rust_package_planning.cargo_free_topology_proof]
  - Evidence: `scripts/prove-cargo-free-rust-plan.sh --check --bundle-dir target/cargo-free-rust-plan-proof/check` and `--full --bundle-dir target/cargo-free-rust-plan-proof/full` both passed.
- [x] [serial] Add Cargo-forbidden guard/shim and record it in proof evidence. r[rust_package_planning.cargo_free_topology_proof]
  - Evidence: proof full mode passes a `cargo-forbidden` shim to `rust-plan --no-cargo-oracle`; `target/cargo-free-rust-plan-proof/full/meta.json` records `cargo_forbidden_marker_absent=true`.
- [x] [serial] Run native Cargo-free planner/executor over the selected workspace. r[rust_package_planning.cargo_free_topology_proof]
  - Evidence: `target/cargo-free-rust-plan-proof/full/receipt.json` reports `rust_plan.cargo_mode.no_cargo_oracle=true`, `topology_execution.execution_status=success`, and 3 unit executions for the generated path workspace.
- [x] [serial] Write durable audit bundle with receipts, streams, source/tool identities, outputs, and blocker summaries. r[rust_package_planning.cargo_free_topology_proof]
  - Evidence: proof bundle contains `preflight.json`, `receipt.json`, `stderr.txt`, `status.txt`, `output-digests.json`, `smoke-stdout.txt`, `smoke-stderr.txt`, and `meta.json`; `evidence/verification.md` records the bounded non-self-build decision.
- [x] [serial] Add final binary smoke check and output digest verification. r[rust_package_planning.cargo_free_topology_proof]
  - Evidence: `target/cargo-free-rust-plan-proof/full/meta.json` records the produced proof-fixture executable path and `smoke_stdout=42`; `output-digests.json` is populated from the receipt; `evidence/verification.md` states this is not Mantle/Crunch self-build evidence.

## Verification

- [x] [serial] Run proof preflight, negative Cargo-shim test, full ignored proof or documented blocker run, and Cairn validation. r[rust_package_planning.cargo_free_topology_proof]
  - Evidence: `cargo test -p mantle --test rust_plan_cli rust_plan_cli_no_cargo_oracle_executes_path_workspace_without_invoking_cargo -- --nocapture` passed; `scripts/prove-cargo-free-rust-plan.sh --check --bundle-dir target/cargo-free-rust-plan-proof/check` passed; `scripts/prove-cargo-free-rust-plan.sh --full --bundle-dir target/cargo-free-rust-plan-proof/full` passed; `cairn validate --root .` returned `valid: true`.
