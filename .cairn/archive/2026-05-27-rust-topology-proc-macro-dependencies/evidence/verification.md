# Verification

## Baseline

Before this change, focused existing proc-macro topology tests passed:

- `cargo test -p mantle --bin mantle host_dependency_topology_accepts_proc_macro_host_producer -- --nocapture`
  - Result: `1 passed; 0 failed`.
- `cargo test -p mantle --bin mantle unit_derivation_graph_binds_proc_macro_host_artifacts_to_target_units -- --nocapture`
  - Result: `1 passed; 0 failed`.

The pre-change self-probe frontier was `async-stream-impl` failing with unresolved `proc_macro` / `proc_macro2` / `quote` / `syn` imports.

## Focused tests after implementation

Commands run after implementation commit `0e17e26b`:

- `cargo test -p mantle --bin mantle native_unit_derivation_adds_selected_feature_cfg_args -- --nocapture`
  - Result: `1 passed; 0 failed`.
- `cargo test -p mantle --bin mantle native_host_derivation_adds_compiler_proc_macro_extern -- --nocapture`
  - Result: `1 passed; 0 failed`.
- `cargo test -p mantle --bin mantle native_host_dependencies_use_normal_deps_only_for_proc_macro_units -- --nocapture`
  - Result: `1 passed; 0 failed`.
- `cargo test -p mantle --bin mantle combined_unit_topology_orders_proc_macro_dependency_lib_before_host_unit -- --nocapture`
  - Result: `1 passed; 0 failed`.

## Lifecycle gates

After updating the change to include feature cfg propagation:

- `nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle`: passed.
- `nix run path:/home/brittonr/git/cairn#cairn -- gate proposal rust-topology-proc-macro-dependencies --root /home/brittonr/git/mantle`: PASS.
- `nix run path:/home/brittonr/git/cairn#cairn -- gate design rust-topology-proc-macro-dependencies --root /home/brittonr/git/mantle`: PASS.
- `nix run path:/home/brittonr/git/cairn#cairn -- gate tasks rust-topology-proc-macro-dependencies --root /home/brittonr/git/mantle`: PASS.

## Self-probe oracle checkpoint

- **Question:** Does committed implementation HEAD `0e17e26b` move the proc-macro frontier past the old `async-stream-impl` unresolved `proc_macro` / dependency-import failure while preserving clean-tree evidence?
- **Inspected evidence:** pueue task `47`; `target/mantle-self-rust-plan-probe-after-0e17e26b-clean/head.txt`; `target/mantle-self-rust-plan-probe-after-0e17e26b-clean/git-status-short.txt`; `target/mantle-self-rust-plan-probe-after-0e17e26b-clean/receipt.json`; `target/mantle-self-rust-plan-probe-after-0e17e26b-clean/blocker-summary.txt`.
- **Decision:** Yes. The probe is tied to clean implementation commit `0e17e26b`, `async-stream-impl` executes successfully as a proc-macro host unit, its derivation carries selected dependencies (`proc_macro2`, `quote`, `syn`) and compiler `proc_macro`, and `syn` receives selected feature cfgs including `parsing` and `visit-mut`. The topology advances from 5 to 15 unit executions and 3 to 6 metadata runs. The next deterministic blocker is `aws-lc-rs` build-script metadata requiring `DEP_AWS_LC_...` include env.
- **Owner:** coding agent.
- **Next action:** create a separate bounded change for build-script dependency metadata/env propagation needed by `aws-lc-rs`.

## Final-head evidence addendum

After archive and agent-note commits, pueue task `56` reran the same self-probe at final code-bearing HEAD `573bc89b`. Later evidence-only commits do not modify `src/rust_plan.rs` or accepted specs. Task `56` confirms the same decision at `573bc89b`: clean tree, `async-stream-impl` proc-macro success, 15 topology unit executions, 6 metadata runs, next blocker at `aws-lc-rs` build-script include env.

A follow-up transcript also ran `git diff --check`; it produced no output, so whitespace checks passed.

Final code-bearing HEAD probe excerpt from task `56`:

```text
probe: target/mantle-self-rust-plan-probe-after-573bc89b-clean/receipt.json
head: 573bc89b43746188f10fd5f0c4c5d447fa287da9
git_status_short_bytes=0

probe_status=0
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

Implementation checkpoint excerpt:

```text
probe: target/mantle-self-rust-plan-probe-after-0e17e26b-clean/receipt.json
head: 0e17e26b9c04e01fe5e9d8b0ae28997f717c272a
git_status_short_bytes=0

probe_status=0
topology_execution=blocked
topology_unit_executions=15
metadata_runs=6

async-stream-impl unit execution:
registry+https://github.com/rust-lang/crates.io-index#async-stream-impl@0.3.6 target=proc-macro status=success reason=

async-stream-impl derivation deps/features:
target=proc-macro deps=proc_macro2,quote,syn proc_macro_extern=true
syn_cfgs=feature="clone-impls",feature="default",feature="derive",feature="extra-traits",feature="fold",feature="full",feature="parsing",feature="printing",feature="proc-macro",feature="visit",feature="visit-mut"

metadata packages:
registry+https://github.com/rust-lang/crates.io-index#anyhow@1.0.102 status=success
registry+https://github.com/rust-lang/crates.io-index#assert_cmd@2.2.0 status=success
registry+https://github.com/rust-lang/crates.io-index#async-io@2.6.0 status=success
registry+https://github.com/rust-lang/crates.io-index#proc-macro2@1.0.106 status=success
registry+https://github.com/rust-lang/crates.io-index#quote@1.0.45 status=success
registry+https://github.com/rust-lang/crates.io-index#atomic-polyfill@1.0.3 status=success

blocker classes:
      1 build-script-run-failed

topology blocker:
- build-script-run-failed:
thread 'main' (663935) panicked at /home/brittonr/git/mantle/vendor-deps/aws-lc-rs/build.rs:110:5:
missing DEP_AWS_LC_ include
```
