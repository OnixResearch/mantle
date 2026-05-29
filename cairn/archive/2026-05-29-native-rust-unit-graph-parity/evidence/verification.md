# Verification checkpoint: native-rust-unit-graph-parity

Question: Is the archived native Rust unit graph parity change supported by durable evidence beyond ephemeral pueue task IDs?

Inspected evidence:
- `target/mantle-self-rust-plan-probe-unit-graph-parity-post10/blocker-summary.txt` existed during repair and reported: `package_ready=true`, `unit_ready=true`, `host_ready=true`, `native_units=485`, `topology_execution_status=success`, `executions=580`, `metadata_runs=59`, and no blocker class/message.
- Focused verification was rerun during the archive session: `cargo test -p mantle --bin mantle rust_plan::tests::native_unit_graph -- --nocapture` reported 12 passed.
- Focused verification was rerun during the archive session: `cargo test -p mantle --bin mantle rust_plan::tests::native_host -- --nocapture` reported 16 passed.
- `cairn validate --root .` reported `valid: true` after the archive repair.

Decision: Treat the archived change as supported by the recorded receipt summary and focused reruns. Do not rely on bare pueue task IDs as the only proof in future task evidence.

Owner: Mantle agent.

Next action: If this evidence is challenged again, rerun the focused commands and regenerate the self-probe receipt summary under `target/mantle-self-rust-plan-probe-unit-graph-parity-post10/` or a new numbered directory.
