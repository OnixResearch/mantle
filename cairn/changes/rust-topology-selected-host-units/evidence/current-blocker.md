# Current Blocker

Task-ID: H1
Covers: rust_package_planning.native_selected_host_units

## Question

What deterministic frontier remains after bounded link-lib metadata parsing?

## Inspected evidence

Clean probe receipt from prior completed change:

- Receipt: `target/mantle-self-rust-plan-probe-after-1c1952e1-clean/receipt.json`.
- Topology status: `blocked`.
- `aws-lc-sys` and `aws-lc-rs` build-script metadata runs reported `success`.
- Remaining blocker: `rustc-failed` compiling `jiff-static@0.2.23` as a host proc-macro with unresolved `quote` and `syn` imports.
- Cargo `--unit-graph` for the current workspace does not include `jiff-static`; it appears only in package metadata/lock material.

## Decision

Treat this as a selected-host-unit planning boundary. Native host planning must follow Cargo-selected host units, not every manifest-visible host target.

## Owner

Mantle agent.

## Next action

Add tests, filter native host units by Cargo-selected host facts, then rerun focused validation plus a clean self-probe.
