# Verification

## Review oracle checkpoint

See `oracle-checkpoint.md` for human-route review evidence tying the implementation/test changes to commit `7f5e6da4`, the archive/spec sync to commit `87a7e328`, the push transcript, and the manual `1970-01-01` archive rename to `2026-05-28`.

## Baseline

Task-ID: V0
Covers: rust_package_planning.native_self_package_lib_binding

Command: pueue task `169` (`baseline-self-package-combined-topology-tests`).

Result: existing combined topology tests passed before implementation:

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 523 filtered out; finished in 0.00s
```

## Focused tests

Task-ID: V1
Covers: rust_package_planning.native_self_package_lib_binding

Commands: pueue task `175` (`self-package-focused-tests-rerun`).

```sh
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
export CARGO_TARGET_DIR=/tmp/mantle-self-package-tests
cargo test -p mantle --bin mantle native_unit_graph_filters_package_self_dependency_from_lib_unit -- --nocapture
cargo test -p mantle --bin mantle combined_unit_topology -- --nocapture
```

Result: new self-package regression passed; combined topology suite passed 7/7:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 530 filtered out; finished in 0.00s
...
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 524 filtered out; finished in 0.00s
```

## Native unit graph suite

Task-ID: V1b
Covers: rust_package_planning.native_self_package_lib_binding

Command: pueue task `180` (`self-package-native-unit-graph-tests`).

Result: the broader native unit graph suite passed, including positive external dependency retention and negative missing-producer/source cases:

```text
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 522 filtered out; finished in 0.00s
```

## Dirty topology probe

Task-ID: V2
Covers: rust_package_planning.native_self_package_lib_binding

Command: pueue task `179` (`self-package-dirty-self-probe`).

Summary from `target/mantle-self-rust-plan-probe-self-package-dirty/blocker-summary.txt`:

```text
probe: target/mantle-self-rust-plan-probe-self-package-dirty/receipt.json
head: 10537c72c2bc05690bd86385b220084e8cf0c4cf
git_status_short_bytes=38
probe_status=0
topology_status=null
executions=602
blocker={"class":"rustc-failed","message":"error[E0463]: can't find crate for `crunch_glue`\n --> /home/brittonr/git/mantle/crates/crunch-system/src/assembler.rs:4:5\n  |\n4 | use crunch_glue::CrunchDerivation;\n  |     ^^^^^^^^^^^ can't find crate\n\nerror: aborting due to 1 previous error\n\nFor more information about this error, try `rustc --explain E0463`."}
```

Receipt check using corrected field names:

```sh
jq -r 'keys[], .topology_execution.execution_status, (.topology_execution.unit_executions|length), .topology_execution.blocker.class' target/mantle-self-rust-plan-probe-self-package-dirty/receipt.json
```

Result: the prior root `mantle` same-package self-dependency internal error is gone. The topology executes 602 units and reaches the next deterministic frontier: `crunch-system` cannot find selected normal dependency crate `crunch_glue`.

## Clean topology probe

Task-ID: V2b
Covers: rust_package_planning.native_self_package_lib_binding

Command: pueue task `181` launched the clean probe after commit `7f5e6da4055d5183c1922d297b62f119934ad221`; its shell summary trailer was incomplete, so the task itself failed after the probe wrote `target/mantle-self-rust-plan-probe-self-package-clean/receipt.json`. The checked summary was generated afterward from that receipt.

Summary from `target/mantle-self-rust-plan-probe-self-package-clean/blocker-summary.txt`:

```text
probe: target/mantle-self-rust-plan-probe-self-package-clean/receipt.json
head: 7f5e6da4055d5183c1922d297b62f119934ad221
git_status_short_bytes=0
probe_status=0
topology_execution_status=blocked
executions=602
blocker={"class":"rustc-failed","message":"error[E0463]: can't find crate for `crunch_glue`\n --> /home/brittonr/git/mantle/crates/crunch-system/src/assembler.rs:4:5\n  |\n4 | use crunch_glue::CrunchDerivation;\n  |     ^^^^^^^^^^^ can't find crate\n\nerror: aborting due to 1 previous error\n\nFor more information about this error, try `rustc --explain E0463`."}
```

Result: clean committed tree also passes the self-package frontier and reaches the same next deterministic `crunch_glue` binding frontier.

## Static checks

Task-ID: V3
Covers: rust_package_planning.native_self_package_lib_binding

Commands:

```sh
git diff --check
cargo fmt --check -p mantle -- src/rust_plan.rs
```

Result: both commands passed.

## Cairn validation and gates

Task-ID: V4
Covers: rust_package_planning.native_self_package_lib_binding

Commands:

```sh
/home/brittonr/.cargo-target/debug/cairn validate --root .
/home/brittonr/.cargo-target/debug/cairn gate proposal rust-topology-self-package-lib-binding --root .
/home/brittonr/.cargo-target/debug/cairn gate design rust-topology-self-package-lib-binding --root .
/home/brittonr/.cargo-target/debug/cairn gate tasks rust-topology-self-package-lib-binding --root .
```

Result before final evidence/task edits: `valid: true`, `changes: 1`, `specs_validated: 2`; proposal/design/tasks gates all returned `verdict: PASS`.
