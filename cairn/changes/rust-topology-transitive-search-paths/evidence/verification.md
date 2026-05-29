# Verification

## Focused regression test

Task-ID: V1
Covers: rust_package_planning.native_transitive_search_paths

Command:

```sh
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
export CARGO_TARGET_DIR=/tmp/mantle-transitive-search-focused
cargo test -p mantle --bin mantle append_all_produced_dependency_search_paths_keeps_duplicate_package_variant_dirs -- --nocapture
```

Result:

```text
test rust_plan::tests::append_all_produced_dependency_search_paths_keeps_duplicate_package_variant_dirs ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 531 filtered out; finished in 0.00s
```

The test covers the positive path (two same-package variant parent directories are both emitted as `-L dependency`) and the negative/dedup path (a pre-existing search path is not duplicated; final dependency-search set length is exactly the expected count).

## Native unit graph suite

Task-ID: V2
Covers: rust_package_planning.native_transitive_search_paths

Command:

```sh
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
export CARGO_TARGET_DIR=/tmp/mantle-transitive-search-native-tests
cargo test -p mantle --bin mantle native_unit_graph -- --nocapture
```

Result:

```text
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 523 filtered out; finished in 0.00s
```

## Dirty topology probe

Task-ID: V3
Covers: rust_package_planning.native_transitive_search_paths

Command: pueue task `193` (`transitive-search-clean-self-probe`) while the implementation was in the dirty worktree.

Summary from `target/mantle-self-rust-plan-probe-transitive-search-clean/blocker-summary.txt`:

```text
probe: target/mantle-self-rust-plan-probe-transitive-search-clean/receipt.json
head: 0ba3aff2bdaeeaad94ccffc60358f98f3c56e86f
git_status_short_bytes=38
probe_status=0
topology_execution_status=blocked
executions=611
blocker={"class":"rustc-failed","message":"error[E0463]: can't find crate for `crunch_store`\n --> ./crates/crunch-build/src/ca_mapping.rs:5:9\n  |\n5 | pub use crunch_store::CaMappings;\n  |         ^^^^^^^^^^^^ can't find crate\n\nerror: found crates (`snix_castore` and `snix_castore`) with colliding StableCrateId values\n --> ./crates/crunch-build/src/fod.rs:5:5\n  |\n5 | use snix_castore::Node;\n  |     ^^^^^^^^^^^^\n"}
```

Result: the prior `crunch-system` / `crunch_glue` blocker is gone. The dirty probe advanced from 602 to 611 executed units and reached the next deterministic frontier in `crunch-build`: missing `crunch_store` plus colliding `snix_castore` crate identities.

## Static checks and Cairn gates

Task-ID: V4
Covers: rust_package_planning.native_transitive_search_paths

Commands:

```sh
cargo fmt --check -p mantle -- src/rust_plan.rs
git diff --check
/home/brittonr/.cargo-target/debug/cairn validate --root .
/home/brittonr/.cargo-target/debug/cairn gate proposal rust-topology-transitive-search-paths --root .
/home/brittonr/.cargo-target/debug/cairn gate design rust-topology-transitive-search-paths --root .
/home/brittonr/.cargo-target/debug/cairn gate tasks rust-topology-transitive-search-paths --root .
```

Result: static checks passed before writing this evidence; Cairn validation/gates are rerun after evidence updates before archive.
