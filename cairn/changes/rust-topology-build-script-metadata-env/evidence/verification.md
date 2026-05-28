# Verification

## Focused tests

Pueue task `70` ran:

```sh
cargo test -p mantle --bin mantle rust_plan::
```

with the documented Mantle Rust toolchain/linker environment. Result: `test result: ok. 66 passed; 0 failed; 0 ignored; 0 measured; 427 filtered out; finished in 0.04s`.

Coverage added for this change includes:

- positive package-level `[package] build = "builder/main.rs"` planning and `links` capture;
- negative `[package] build = false` custom-build suppression;
- positive custom `cargo:include=...` metadata capture;
- negative malformed custom metadata key rejection;
- positive `DEP_AWS_LC_0_39_1_INCLUDE` env construction;
- positive linked metadata host ordering;
- negative missing linked metadata producer blocker.

## Lifecycle gates

Commands run after implementation and spec/design wording updates:

```sh
/home/brittonr/.cargo-target/debug/cairn validate --root .
/home/brittonr/.cargo-target/debug/cairn gate proposal rust-topology-build-script-metadata-env --root .
/home/brittonr/.cargo-target/debug/cairn gate design rust-topology-build-script-metadata-env --root .
/home/brittonr/.cargo-target/debug/cairn gate tasks rust-topology-build-script-metadata-env --root .
git diff --check
```

Results: validate/gates passed; `git diff --check` produced no output.

## Self-probe oracle checkpoint

- **Question:** Does linked build-script metadata propagation move native topology past the prior `aws-lc-rs` `missing DEP_AWS_LC_ include` blocker?
- **Inspected evidence:** pueue task `68`; `target/mantle-self-rust-plan-probe-after-build-script-metadata-env-dirty/receipt.json`; `target/mantle-self-rust-plan-probe-after-build-script-metadata-env-dirty/blocker-summary.txt`.
- **Decision:** Yes for the implementation tree. Topology advanced from 15 to 30 unit executions and from 6 to 12 metadata runs. `aws-lc-rs` no longer stops at missing `DEP_AWS_LC_..._INCLUDE`; ordering now reaches `aws-lc-sys` custom-build compilation first. The new deterministic frontier is `aws-lc-sys` build-script rustc failure because `CARGO_PKG_VERSION` is not defined at compile time.
- **Owner:** coding agent.
- **Next action:** handle `CARGO_PKG_VERSION` in a follow-up change.

## Final clean-head self-probe

Pueue task `73` reran the topology self-probe after implementation commit `089a1b5a` with a clean worktree. It confirmed the same frontier movement: the previous `aws-lc-rs` `missing DEP_AWS_LC_ include` blocker is gone, `aws-lc-sys` is now reached first, and the remaining deterministic blocker is compile-time `CARGO_PKG_VERSION` for the linked dependency build script.

Clean probe excerpt:

```text
probe: target/mantle-self-rust-plan-probe-after-089a1b5a-clean/receipt.json
head: 089a1b5a11959ff3ae8c6297b174499c214c97b7
git_status_short_bytes=0

probe_status=0
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

Dirty implementation probe excerpt:

```text
probe: target/mantle-self-rust-plan-probe-after-build-script-metadata-env-dirty/receipt.json
head: fa976a59ea3e5928fd7029e6acae3a967d6ea886
git_status_short_bytes=20

probe_status=0
topology_execution=blocked
topology_unit_executions=30
metadata_runs=12

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
