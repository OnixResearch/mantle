# Current Blocker

Task-ID: H1
Covers: rust_package_planning.native_manifest_links_env

## Question

What deterministic frontier remains after selected host-unit planning?

## Inspected evidence

Clean probe receipt from selected host-unit implementation:

- Receipt: `target/mantle-self-rust-plan-probe-after-21bd9c17-clean/receipt.json`.
- HEAD: `21bd9c174225d136131c10a12494384d541b291e`.
- `git_status_short_bytes=0`.
- Topology status: `blocked`.
- `jiff-static` executions: none.
- `aws-lc-sys` and `aws-lc-rs` metadata runs: `success`.
- Remaining blocker: `build-script-run-failed` in `vendor-deps/ring/build.rs`, panic at line 287 from `Option::unwrap()` on `None`.
- Inspected `vendor-deps/ring/build.rs`: line 287 unwraps `env::var("CARGO_MANIFEST_LINKS")` and asserts it equals the package links-derived core name.

## Decision

Treat this as bounded package-env parity. Native build-script env must include `CARGO_MANIFEST_LINKS` from the package manifest `links` field.

## Owner

Mantle agent.

## Next action

Add package-env tests, implement the env key, then rerun focused validation plus a clean self-probe.
