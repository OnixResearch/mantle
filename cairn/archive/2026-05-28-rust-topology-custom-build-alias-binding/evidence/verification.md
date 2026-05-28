# Verification

Focused tests:

```sh
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
export CARGO_TARGET_DIR=/tmp/mantle-custom-build-alias-test
cargo test -p mantle --bin mantle bind_all_host_artifacts -- --nocapture
```

Results:

- `bind_all_host_artifacts`: `5 passed`.
- Included positive coverage for `build_script_main` placeholder rewrite.
- Included negative coverage proving unknown same-package custom-build aliases stay unbound.
- Included negative coverage proving custom-build host artifacts do not synthesize proc-macro extern flags.

Dirty self-probe:

```sh
pueue_log 77
```

Result summary from `target/mantle-self-rust-plan-probe-custom-build-alias-dirty/blocker-summary.txt`:

- `probe_status=0`.
- `topology_execution=blocked`.
- `topology_unit_executions=352`.
- `metadata_runs=63`.
- `aws-lc-sys@0.39.1 custom-build status=success`.
- `aws-lc-sys@0.39.1 lib status=success`.
- Next frontier: `rustc-failed` in `vendor-deps/spez/src/lib.rs` because `#[proc_macro]` was compiled without proc-macro crate type.

Clean self-probe after commit `e338baec`:

```sh
pueue_log 80
```

Result summary from `target/mantle-self-rust-plan-probe-after-e338baec-clean/blocker-summary.txt`:

- `git_status_short_bytes=0`.
- `probe_status=0`.
- `topology_execution=blocked`.
- `topology_unit_executions=352`.
- `metadata_runs=63`.
- `aws-lc-sys@0.39.1 custom-build status=success`.
- `aws-lc-sys@0.39.1 lib status=success`.
- Next frontier unchanged: `rustc-failed` in `vendor-deps/spez/src/lib.rs` because `#[proc_macro]` was compiled without proc-macro crate type.

Cairn sync, validation, and tasks gate:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- sync rust-topology-custom-build-alias-binding --root . --execute
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks rust-topology-custom-build-alias-binding --root .
```

Results:

- `sync`: `blocked=false`, `mutated=true`.
- `validate`: `valid=true`, `changes=1`, `specs_validated=2`.
- `gate tasks`: `valid=true`, `verdict=PASS`.
